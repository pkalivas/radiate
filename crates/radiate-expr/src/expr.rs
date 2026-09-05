use crate::ops::{BinaryOp, RollupOp, ScheduleOp, SelectOp, TrinaryOp, UnaryOp};
use crate::{ExprResult, ProjectExpr};
use radiate_utils::{AnyValue, SmallStr};
use radiate_utils::{WindowBuffer, sentry_id};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};

sentry_id!(ExprId);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ExprNode {
    Literal(AnyValue<'static>),
    Selector(SelectOp),
    Schedule(ScheduleOp),
    Rolling {
        child: Box<Expr>,
        buffer: WindowBuffer<AnyValue<'static>>,
    },
    Reduce {
        child: Box<Expr>,
        rollup: RollupOp,
    },
    Unary {
        child: Box<Expr>,
        op: UnaryOp,
    },
    Binary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinaryOp,
    },
    Trinary {
        first: Box<Expr>,
        second: Box<Expr>,
        third: Box<Expr>,
        op: TrinaryOp,
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub struct Expr {
    pub(crate) name: SmallStr,
    pub(crate) id: ExprId,
    pub(crate) node: ExprNode,
}

impl Expr {
    pub fn new(node: ExprNode) -> Self {
        let id = ExprId::new();
        Self {
            name: SmallStr::from_string(format!("Expr<{:?}>", id.get())),
            id,
            node,
        }
    }

    pub fn id(&self) -> ExprId {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn alias(mut self, name: impl Into<SmallStr>) -> Self {
        self.name = name.into();
        self
    }

    pub fn is_literal(&self) -> bool {
        matches!(self.node, ExprNode::Literal(_))
    }

    pub fn is_selector(&self) -> bool {
        matches!(self.node, ExprNode::Selector(_))
    }

    pub fn is_schedule(&self) -> bool {
        matches!(self.node, ExprNode::Schedule(_))
    }

    pub fn into_schedule(self) -> Option<Expr> {
        if self.is_schedule() {
            return Some(self);
        }

        if let ExprNode::Literal(value) = &self.node {
            if value.is_numeric() {
                return value
                    .extract::<usize>()
                    .map(|interval| Expr::every(interval).into());
            } else if value.is_duration() {
                return value
                    .extract::<f32>()
                    .map(|seconds| Expr::throttle(Duration::from_secs_f32(seconds)).into());
            }
        }

        Some(self)
    }

    pub fn walk(&self, f: &mut impl FnMut(&Expr)) {
        f(self);
        for child in self.children() {
            child.walk(f);
        }
    }

    fn children(&self) -> Vec<&Expr> {
        match &self.node {
            ExprNode::Literal(_) | ExprNode::Selector(_) | ExprNode::Schedule(_) => vec![],
            ExprNode::Rolling { child: r, .. } => vec![&r],
            ExprNode::Reduce { child: r, .. } => vec![&r],
            ExprNode::Unary { child: u, .. } => vec![&u],
            ExprNode::Binary { lhs, rhs, .. } => vec![&lhs, &rhs],
            ExprNode::Trinary {
                first,
                second,
                third,
                ..
            } => vec![&first, &second, &third],
        }
    }
}

impl Expr {
    #[inline]
    pub fn trigger(&mut self) -> ExprResult<'static> {
        self.evaluate(&AnyValue::Null).map(|val| val.into_static())
    }

    #[inline]
    pub fn evaluate<'a>(&'a mut self, input: &'a impl ProjectExpr<'a>) -> ExprResult<'a> {
        match &mut self.node {
            ExprNode::Literal(value) => Ok(value.clone()),
            ExprNode::Selector(selector) => input.select(selector),
            ExprNode::Schedule(op) => super::eval::try_schedule(op),

            ExprNode::Rolling { child, buffer } => super::eval::rolling_eval(child, input, buffer),
            ExprNode::Reduce { child, rollup } => super::eval::reduce_eval(child, input, rollup),

            ExprNode::Unary { child, op } => super::eval::unary_eval(child, op, input),
            ExprNode::Binary { lhs, rhs, op } => super::eval::binary_eval(lhs, rhs, op, input),
            ExprNode::Trinary {
                first,
                second,
                third,
                op,
            } => super::eval::trinary_eval(first, second, third, op, input),
        }
    }
}

impl<'a> From<AnyValue<'a>> for Expr {
    fn from(value: AnyValue<'a>) -> Self {
        Expr::new(ExprNode::Literal(value.into_static()))
    }
}

impl From<SelectOp> for Expr {
    fn from(selector: SelectOp) -> Self {
        Expr::new(ExprNode::Selector(selector))
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_literal() {
        let expr = Expr::select("one")
            .rolling(10)
            .mean()
            .div(10 as f32)
            .clamp(1.0_f32, 5.0_f32);

        fn print(expr: &Expr, depth: usize) {
            println!("{}{} {:?}", "  ".repeat(depth), expr.name(), expr);
            for child in expr.children() {
                print(child, depth + 1);
            }
        }

        print(&expr, 0);
    }
}
