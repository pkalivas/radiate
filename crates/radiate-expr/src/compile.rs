use crate::ops::{BinaryOp, UnaryOp};
use crate::{Expr, ExprNode};
use radiate_utils::AnyValue;

impl Expr {
    /// Walks the tree bottom-up and rewrites algebraically equivalent shapes
    /// into the smallest possible form, without changing what the expression
    /// evaluates to (types included, up to float rounding). Specifically:
    ///
    /// - Pure-literal `+ - * / %` folds using the evaluator's own `AnyValue`
    ///   arithmetic (`Lit(2_i32) + Lit(3_i32)` → `Lit(5_i32)`)
    /// - `Add` / `Sub` / `Mul` / `Div` with one *float* literal operand fuses
    ///   into a `Unary(Affine)` (`x * 5.0 + 3.0` → `Affine { scale: 5, bias: 3 }`).
    ///   Integer literals never fuse, so integer arithmetic is preserved.
    /// - Nested affines collapse: `s2 * (s1*x + b1) + b2` → `Affine(s2*s1, s2*b1 + b2)`
    ///
    /// One deliberate gap: a Duration multiplied by a chain whose running
    /// factor goes negative and back (`d * -1.0 * -1.0`) is `Null` unfused but
    /// a Duration once fused.
    pub fn compile(self) -> Expr {
        let name = self.name;
        let kind = compile_kind(self.node);
        let id = self.id;
        Expr {
            name,
            id,
            node: kind,
        }
    }
}

fn compile_kind(kind: ExprNode) -> ExprNode {
    match kind {
        ExprNode::Literal(_) | ExprNode::Selector(_) | ExprNode::Schedule(_) => kind,

        ExprNode::Unary { child: u, op } => {
            let child = u;
            let child = child.compile();
            match op {
                UnaryOp::Affine { scale, bias, wide } => fuse_affine(child, scale, bias, wide).node,
                other_op => ExprNode::Unary {
                    child: Box::new(child),
                    op: other_op,
                },
            }
        }

        ExprNode::Trinary {
            first,
            second,
            third,
            op,
        } => ExprNode::Trinary {
            first: Box::new((*first).compile()),
            second: Box::new((*second).compile()),
            third: Box::new((*third).compile()),
            op,
        },

        ExprNode::Binary {
            lhs: lhs_box,
            rhs: rhs_box,
            op,
        } => {
            let lhs = (*lhs_box).compile();
            let rhs = (*rhs_box).compile();
            reduce_binary(lhs, rhs, op).node
        }

        ExprNode::Reduce { mut child, rollup } => {
            let old_child = std::mem::replace(
                &mut child,
                Box::new(Expr::new(ExprNode::Literal(AnyValue::Null))),
            );
            let new_child = old_child.compile();
            ExprNode::Reduce {
                child: Box::new(new_child),
                rollup,
            }
        }
        ExprNode::Rolling { mut child, buffer } => {
            let old_child = std::mem::replace(
                &mut child,
                Box::new(Expr::new(ExprNode::Literal(AnyValue::Null))),
            );
            let new_child = old_child.compile();
            ExprNode::Rolling {
                child: Box::new(new_child),
                buffer,
            }
        }
    }
}

fn reduce_binary(lhs: Expr, rhs: Expr, op: BinaryOp) -> Expr {
    if let (ExprNode::Literal(l), ExprNode::Literal(r)) = (&lhs.node, &rhs.node)
        && let Some(folded) = fold_literals(l, r, op)
    {
        return Expr::new(ExprNode::Literal(folded));
    }

    // Only *float* literals fuse. Under AnyValue promotion, anything combined
    // with a float becomes a float (or a scaled Duration), which is exactly
    // what Affine computes. Integer literals stay as Binary nodes so integer
    // arithmetic (and its truncating division) is preserved.
    let float_lit = |e: &Expr| match &e.node {
        ExprNode::Literal(v @ (AnyValue::Float32(_) | AnyValue::Float64(_))) => v
            .extract::<f64>()
            .map(|k| (k, matches!(v, AnyValue::Float64(_)))),
        _ => None,
    };

    match op {
        BinaryOp::Add => {
            if let Some((k, wide)) = float_lit(&rhs) {
                return fuse_affine(lhs, 1.0, Some(k), wide);
            }
            if let Some((k, wide)) = float_lit(&lhs) {
                return fuse_affine(rhs, 1.0, Some(k), wide);
            }
        }
        BinaryOp::Sub => {
            if let Some((k, wide)) = float_lit(&rhs) {
                return fuse_affine(lhs, 1.0, Some(-k), wide);
            }
            if let Some((k, wide)) = float_lit(&lhs) {
                return fuse_affine(rhs, -1.0, Some(k), wide);
            }
        }
        BinaryOp::Mul => {
            if let Some((s, wide)) = float_lit(&rhs) {
                return fuse_affine(lhs, s, None, wide);
            }
            if let Some((s, wide)) = float_lit(&lhs) {
                return fuse_affine(rhs, s, None, wide);
            }
        }
        BinaryOp::Div => {
            // A zero divisor stays unfused so it yields Null like the
            // unfused path, instead of an infinite scale.
            if let Some((d, wide)) = float_lit(&rhs)
                && d != 0.0
                && d.is_finite()
            {
                return fuse_affine(lhs, 1.0 / d, None, wide);
            }
        }
        _ => {}
    }

    Expr::new(ExprNode::Binary {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        op,
    })
}

/// Fold two literals using the same `AnyValue` arithmetic the evaluator uses,
/// so the folded literal has exactly the type and value evaluation would give.
fn fold_literals(
    l: &AnyValue<'static>,
    r: &AnyValue<'static>,
    op: BinaryOp,
) -> Option<AnyValue<'static>> {
    let (l, r) = (l.clone(), r.clone());
    match op {
        BinaryOp::Add => Some(l + r),
        BinaryOp::Sub => Some(l - r),
        BinaryOp::Mul => Some(l * r),
        BinaryOp::Div => Some(l / r),
        BinaryOp::Mod => Some(l % r),
        _ => None,
    }
}

/// Construct `Unary(Affine(child * scale + bias))`, collapsing nested affines:
/// `(x * s2 + b2) * scale + bias = x * (scale * s2) + (scale * b2 + bias)`.
fn fuse_affine(child: Expr, scale: f64, bias: Option<f64>, wide: bool) -> Expr {
    if let ExprNode::Unary {
        child: inner,
        op:
            UnaryOp::Affine {
                scale: s2,
                bias: b2,
                wide: w2,
            },
    } = child.node
    {
        let bias = match (b2, bias) {
            (None, None) => None,
            (b2, b) => Some(scale * b2.unwrap_or(0.0) + b.unwrap_or(0.0)),
        };

        return Expr::new(ExprNode::Unary {
            child: inner,
            op: UnaryOp::Affine {
                scale: scale * s2,
                bias,
                wide: wide || w2,
            },
        });
    }

    Expr::new(ExprNode::Unary {
        child: Box::new(child),
        op: UnaryOp::Affine { scale, bias, wide },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn x() -> Expr {
        Expr::identity()
    }

    fn same(a: &AnyValue<'_>, b: &AnyValue<'_>) -> bool {
        use AnyValue::*;
        let close =
            |a: f64, b: f64| (a.is_nan() && b.is_nan()) || (a - b).abs() <= 1e-5 * a.abs().max(1.0);
        match (a, b) {
            (Float32(a), Float32(b)) => close(f64::from(*a), f64::from(*b)),
            (Float64(a), Float64(b)) => close(*a, *b),
            (Duration(a), Duration(b)) => close(a.as_secs_f64(), b.as_secs_f64()),
            _ => a == b,
        }
    }

    /// Compiling must never change what an expression evaluates to.
    #[test]
    fn compiled_matches_uncompiled_across_types() {
        let exprs: Vec<(&str, Expr)> = vec![
            ("x * 2f32", x() * 2.0_f32),
            ("x * 2f32 + 1f32", x() * 2.0_f32 + 1.0_f32),
            (
                "(x - 1f32) * 2f32 + 3f32",
                (x() - 1.0_f32) * 2.0_f32 + 3.0_f32,
            ),
            ("x / 4f32", x() / 4.0_f32),
            ("10f32 - x", Expr::lit(10.0_f32) - x()),
            ("2f32 * x", Expr::lit(2.0_f32) * x()),
            ("x * 2f64", x() * 2.0_f64),
            ("x * 0.5f32 + 1f64", x() * 0.5_f32 + 1.0_f64),
            ("x * 2f32 * 3f32", x() * 2.0_f32 * 3.0_f32),
            ("x / 0f32", x() / 0.0_f32),
            ("x / 2i32", x() / 2_i32),
            ("x + 1i32", x() + 1_i32),
            ("(x + 1i32) * 0.5f32", (x() + 1_i32) * 0.5_f32),
            ("x + 0f32", x() + 0.0_f32),
        ];

        let inputs: Vec<AnyValue<'static>> = vec![
            AnyValue::Float32(3.0),
            AnyValue::Float32(f32::NAN),
            AnyValue::Float64(3.0),
            AnyValue::Int32(7),
            AnyValue::Int32(-7),
            AnyValue::UInt64(7),
            AnyValue::Usize(7),
            AnyValue::Duration(Duration::from_secs_f64(1.5)),
            AnyValue::Null,
            AnyValue::Bool(true),
        ];

        for (label, expr) in &exprs {
            for input in &inputs {
                let mut raw = expr.clone();
                let mut compiled = expr.clone().compile();
                let a = raw.evaluate(input).unwrap().into_static();
                let b = compiled.evaluate(input).unwrap().into_static();
                assert!(
                    same(&a, &b),
                    "{label} on {input:?}: uncompiled {a:?} vs compiled {b:?}"
                );
            }
        }
    }

    #[test]
    fn literal_fold_preserves_type() {
        let e = (Expr::lit(2_i32) + 3_i32).compile();
        assert_eq!(e.node, ExprNode::Literal(AnyValue::Int32(5)));

        let e = (Expr::lit(7_i32) / 2_i32).compile();
        assert_eq!(e.node, ExprNode::Literal(AnyValue::Int32(3)));
    }

    #[test]
    fn int_literal_does_not_fuse() {
        let e = (x() * 2_i32).compile();
        assert!(matches!(e.node, ExprNode::Binary { .. }));
    }

    #[test]
    fn float_chain_fuses_to_single_affine() {
        let e = ((x() - 1.0_f32) * 2.0_f32 + 3.0_f32).compile();
        let ExprNode::Unary { child, op } = e.node else {
            panic!("expected fused affine");
        };
        assert!(child.is_selector());
        assert_eq!(
            op,
            UnaryOp::Affine {
                scale: 2.0,
                bias: Some(1.0),
                wide: false
            }
        );
    }

    #[test]
    fn float64_literal_marks_affine_wide() {
        let mut e = (x() * 2.0_f64).compile();
        let v = e.evaluate(&AnyValue::Float32(1.5)).unwrap().into_static();
        assert_eq!(v, AnyValue::Float64(3.0));
    }
}
