use crate::ops::{BinaryOp, UnaryOp};
use crate::{Expr, ExprNode};
use radiate_utils::AnyValue;

impl Expr {
    /// Walks the tree bottom-up and rewrites algebraically equivalent shapes
    /// into the smallest possible form. Specifically:
    ///
    /// - Pure-literal subtrees fold (`Lit(2) + Lit(3)` → `Lit(5)`)
    /// - `Add` / `Sub` / `Mul` / `Div` with one literal operand fuses into a
    ///   `Unary(Affine)` (`x * 5 + 3` → `Affine { scale: 5, bias: 3 }`)
    /// - Nested affines collapse: `s2 * (s1*x + b1) + b2` → `Affine(s2*s1, s2*b1 + b2)`
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
                UnaryOp::Affine { scale, bias } => fuse_affine(child, scale, bias).node,
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

    match op {
        BinaryOp::Add => {
            if let ExprNode::Literal(v) = &rhs.node
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(lhs, 1.0, k);
            }
            if let ExprNode::Literal(v) = &lhs.node
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(rhs, 1.0, k);
            }
        }
        BinaryOp::Sub => {
            if let ExprNode::Literal(v) = &rhs.node
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(lhs, 1.0, -k);
            }
            if let ExprNode::Literal(v) = &lhs.node
                && let Some(k) = v.extract::<f32>()
            {
                return fuse_affine(rhs, -1.0, k);
            }
        }
        BinaryOp::Mul => {
            if let ExprNode::Literal(v) = &rhs.node
                && let Some(s) = v.extract::<f32>()
            {
                return fuse_affine(lhs, s, 0.0);
            }
            if let ExprNode::Literal(v) = &lhs.node
                && let Some(s) = v.extract::<f32>()
            {
                return fuse_affine(rhs, s, 0.0);
            }
        }
        BinaryOp::Div => {
            if let ExprNode::Literal(v) = &rhs.node
                && let Some(d) = v.extract::<f32>()
                && d != 0.0
                && d.is_finite()
            {
                return fuse_affine(lhs, 1.0 / d, 0.0);
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

/// Construct `Unary(Affine(scale * child + bias))`, collapsing nested affines.
/// `scale * (s2 * x + b2) + bias = (scale * s2) * x + (scale * b2 + bias)`.
///
/// Shared between the `.affine(...)` builder and the compile-pass binary-fusion
/// rewriters so both produce the same fused shape.
fn fuse_affine(child: Expr, scale: f32, bias: f32) -> Expr {
    if let ExprNode::Unary { child: inner, op } = child.node {
        if matches!(op, UnaryOp::Affine { .. }) {
            let UnaryOp::Affine {
                scale: s2,
                bias: b2,
            } = op
            else {
                unreachable!()
            };

            return Expr::new(ExprNode::Unary {
                child: inner,
                op: UnaryOp::Affine {
                    scale: scale * s2,
                    bias: scale * b2 + bias,
                },
            });
        }

        return Expr::new(ExprNode::Unary {
            child: Box::new(Expr::new(ExprNode::Unary { child: inner, op })),
            op: UnaryOp::Affine { scale, bias },
        });
    }

    Expr::new(ExprNode::Unary {
        child: Box::new(child),
        op: UnaryOp::Affine { scale, bias },
    })
}
