use super::{
    Expr,
    ops::{BinaryExpr, BinaryOp, TrinaryExpr, UnaryExpr, UnaryOp, fuse_affine},
};
use radiate_utils::AnyValue;

impl Expr {
    /// Walks the tree bottom-up and rewrites algebraically equivalent shapes
    /// into the smallest possible form. Specifically:
    ///
    /// - Pure-literal subtrees fold (`Lit(2) + Lit(3)` → `Lit(5)`)
    /// - `Add` / `Sub` / `Mul` / `Div` with one literal operand fuses into a
    ///   `Unary(Affine)` (`x * 5 + 3` → `Affine { scale: 5, bias: 3 }`)
    /// - Nested affines collapse: `s2 * (s1*x + b1) + b2` → `Affine(s2*s1, s2*b1 + b2)`
    ///
    /// Called automatically when wrapping in `Rate::Expr` or `NamedExpr`. Safe
    /// to call multiple times — idempotent. Mathematically lossless within
    /// f32 precision (and within the existing arithmetic semantics for Null /
    /// non-finite operands).
    pub fn compile(self) -> Expr {
        match self {
            // Leaves — nothing to rewrite.
            Expr::Literal(_) | Expr::Selector(_) | Expr::Schedule(_) => self,

            Expr::Unary(u) => {
                let UnaryExpr { child, op } = u;
                let child = (*child).compile();
                match op {
                    // Affine on top of a compiled child: re-run fusion so any
                    // newly-revealed Affine nested below collapses upward.
                    UnaryOp::Affine { scale, bias } => fuse_affine(child, scale, bias),
                    other_op => Expr::Unary(UnaryExpr::new(child, other_op)),
                }
            }

            Expr::Trinary(t) => Expr::Trinary(TrinaryExpr::new(
                (*t.first).compile(),
                (*t.second).compile(),
                (*t.third).compile(),
                t.operation,
            )),

            Expr::Binary(b) => {
                let lhs = (*b.lhs).compile();
                let rhs = (*b.rhs).compile();
                reduce_binary(lhs, rhs, b.op)
            }

            // Stateful nodes — keep the rollup/buffer intact, just compile the child.
            Expr::Aggregate(mut a) => {
                let child = std::mem::replace(a.child.as_mut(), Expr::Literal(AnyValue::Null));
                *a.child = child.compile();
                Expr::Aggregate(a)
            }
        }
    }
}

fn reduce_binary(lhs: Expr, rhs: Expr, op: BinaryOp) -> Expr {
    // Pure literal-on-literal: fold to a Literal.
    if let (Expr::Literal(l), Expr::Literal(r)) = (&lhs, &rhs)
        && let Some(folded) = fold_literals(l, r, op)
    {
        return Expr::Literal(folded);
    }

    // Affine fusion patterns. Only Add/Sub/Mul/Div are linear; the rest fall through.
    match op {
        BinaryOp::Add => {
            if let Expr::Literal(v) = &rhs
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(lhs, 1.0, k);
            }
            if let Expr::Literal(v) = &lhs
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(rhs, 1.0, k);
            }
        }
        BinaryOp::Sub => {
            if let Expr::Literal(v) = &rhs
                && let Some(k) = v.extract::<f32>()
            {
                // x - k → Affine(x, 1, -k)
                return fuse_affine(lhs, 1.0, -k);
            }
            if let Expr::Literal(v) = &lhs
                && let Some(k) = v.extract::<f32>()
            {
                // k - x → Affine(x, -1, k)
                return fuse_affine(rhs, -1.0, k);
            }
        }
        BinaryOp::Mul => {
            if let Expr::Literal(v) = &rhs
                && let Some(s) = v.extract::<f32>()
            {
                return fuse_affine(lhs, s, 0.0);
            }
            if let Expr::Literal(v) = &lhs
                && let Some(s) = v.extract::<f32>()
            {
                return fuse_affine(rhs, s, 0.0);
            }
        }
        BinaryOp::Div => {
            // Only fold `x / Lit` (constant divisor). `Lit / x` is non-linear in x.
            if let Expr::Literal(v) = &rhs
                && let Some(d) = v.extract::<f32>()
                && d != 0.0
                && d.is_finite()
            {
                return fuse_affine(lhs, 1.0 / d, 0.0);
            }
        }
        _ => {}
    }

    Expr::Binary(BinaryExpr::new(lhs, rhs, op))
}

fn fold_literals(
    l: &AnyValue<'static>,
    r: &AnyValue<'static>,
    op: BinaryOp,
) -> Option<AnyValue<'static>> {
    let a = l.extract::<f32>()?;
    let b = r.extract::<f32>()?;
    let result = match op {
        BinaryOp::Add => a + b,
        BinaryOp::Sub => a - b,
        BinaryOp::Mul => a * b,
        BinaryOp::Div if b != 0.0 => a / b,
        _ => return None,
    };
    if result.is_finite() {
        Some(AnyValue::Float32(result))
    } else {
        None
    }
}
