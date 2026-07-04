use crate::Expr;
use crate::expr::ExprKind;
use crate::nodes::ops::{BinaryExpr, BinaryOp, TrinaryExpr, UnaryExpr, UnaryOp, fuse_affine};
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
        let name = self.name;
        let kind = compile_kind(self.kind);
        let id = self.id;
        Expr { name, id, kind }
    }
}

fn compile_kind(kind: ExprKind) -> ExprKind {
    match kind {
        ExprKind::Literal(_) | ExprKind::Selector(_) | ExprKind::Schedule(_) => kind,

        ExprKind::Unary(u) => {
            let UnaryExpr { child, op } = u;
            let child = child.compile();
            match op {
                UnaryOp::Affine { scale, bias } => fuse_affine(child, scale, bias).kind,
                other_op => ExprKind::Unary(UnaryExpr::new(child, other_op)),
            }
        }

        ExprKind::Trinary(t) => ExprKind::Trinary(TrinaryExpr::new(
            (*t.first).compile(),
            (*t.second).compile(),
            (*t.third).compile(),
            t.operation,
        )),

        ExprKind::Binary(b) => {
            let lhs = (*b.lhs).compile();
            let rhs = (*b.rhs).compile();
            reduce_binary(lhs, rhs, b.op).kind
        }

        ExprKind::Aggregate(mut a) => {
            let child = std::mem::replace(
                a.child.as_mut(),
                Expr::new(ExprKind::Literal(AnyValue::Null)),
            );
            *a.child = child.compile();
            ExprKind::Aggregate(a)
        }
    }
}

fn reduce_binary(lhs: Expr, rhs: Expr, op: BinaryOp) -> Expr {
    if let (ExprKind::Literal(l), ExprKind::Literal(r)) = (&lhs.kind, &rhs.kind)
        && let Some(folded) = fold_literals(l, r, op)
    {
        return Expr::new(ExprKind::Literal(folded));
    }

    match op {
        BinaryOp::Add => {
            if let ExprKind::Literal(v) = &rhs.kind
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(lhs, 1.0, k);
            }
            if let ExprKind::Literal(v) = &lhs.kind
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(rhs, 1.0, k);
            }
        }
        BinaryOp::Sub => {
            if let ExprKind::Literal(v) = &rhs.kind
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(lhs, 1.0, -k);
            }
            if let ExprKind::Literal(v) = &lhs.kind
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(rhs, -1.0, k);
            }
        }
        BinaryOp::Mul => {
            if let ExprKind::Literal(v) = &rhs.kind
                && let Some(s) = v.extract::<f32>()
            {
                return fuse_affine(lhs, s, 0.0);
            }
            if let ExprKind::Literal(v) = &lhs.kind
                && let Some(s) = v.extract::<f32>()
            {
                return fuse_affine(rhs, s, 0.0);
            }
        }
        BinaryOp::Div => {
            if let ExprKind::Literal(v) = &rhs.kind
                && let Some(d) = v.extract::<f32>()
                && d != 0.0
                && d.is_finite()
            {
                return fuse_affine(lhs, 1.0 / d, 0.0);
            }
        }
        _ => {}
    }

    Expr::new(ExprKind::Binary(BinaryExpr::new(lhs, rhs, op)))
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
