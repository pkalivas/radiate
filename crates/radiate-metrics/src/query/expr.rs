use std::time::Duration;

use crate::{EvalExpr, ExprResult, ExprSelect, nodes::Selector};
use crate::{
    metric_fields,
    nodes::{
        AggExpr, BinaryExpr, IndexState, ScheduleExpr, SelectExpr, TrinaryExpr, UnaryExpr, When,
    },
};
use radiate_utils::sentry_id;
use radiate_utils::{AnyValue, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

sentry_id!(ExprId);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ExprNode {
    Literal(AnyValue<'static>),
    Selector(SelectExpr),
    Aggregate(AggExpr),
    Schedule(ScheduleExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Trinary(TrinaryExpr),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub(crate) name: SmallStr,
    pub(crate) id: ExprId,
    pub(crate) node: ExprNode,
}

impl Expr {
    pub fn new(node: ExprNode) -> Self {
        let id = ExprId::new();
        Self {
            name: SmallStr::from_string(format!("Expr<{:?}>", id)),
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

    pub fn is_aggregate(&self) -> bool {
        matches!(self.node, ExprNode::Aggregate(_))
    }

    pub fn is_schedule(&self) -> bool {
        matches!(self.node, ExprNode::Schedule(_))
    }

    pub fn is_binary(&self) -> bool {
        matches!(self.node, ExprNode::Binary(_))
    }

    pub fn is_unary(&self) -> bool {
        matches!(self.node, ExprNode::Unary(_))
    }

    pub fn is_trinary(&self) -> bool {
        matches!(self.node, ExprNode::Trinary(_))
    }

    pub fn into_schedule(self) -> Option<Expr> {
        if self.is_schedule() {
            return Some(self);
        }

        if let ExprNode::Literal(value) = self.node {
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

        None
    }

    pub fn reset(&mut self) {
        match &mut self.node {
            ExprNode::Literal(_) | ExprNode::Selector(_) => {}
            ExprNode::Aggregate(a) => a.reset(),
            ExprNode::Schedule(s) => s.reset(),
            ExprNode::Binary(b) => {
                b.lhs.reset();
                b.rhs.reset();
            }
            ExprNode::Unary(u) => u.reset(),
            ExprNode::Trinary(t) => {
                t.first.reset();
                t.second.reset();
                t.third.reset();
            }
        }
    }
}

impl Expr {
    pub fn identity() -> Expr {
        Expr::from(SelectExpr::new(Selector::Identity))
    }

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::from(value.into())
    }

    pub fn range(sel: impl Into<std::ops::Range<usize>>) -> Expr {
        Expr::from(SelectExpr::new(sel.into()))
    }

    pub fn metric(name: impl Into<SmallStr>) -> Expr {
        let name = name.into();
        Expr::from(SelectExpr::new(Selector::Nested {
            parent: Box::new(Selector::Field(name)),
            child: Box::new(Selector::Field(metric_fields::LAST_VALUE)),
        }))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::from(ScheduleExpr::Interval(IndexState::new(
            interval,
        ))))
    }

    pub fn throttle(duration: std::time::Duration) -> When {
        When::new(Expr::from(ScheduleExpr::Duration(duration.into())))
    }
}

impl<'a, T> EvalExpr<'a, T> for Expr
where
    T: ExprSelect<'a>,
{
    #[inline]
    fn evaluate(&'a mut self, input: &'a T) -> ExprResult<'a> {
        match &mut self.node {
            ExprNode::Literal(value) => Ok(value.clone()),
            ExprNode::Selector(selector) => selector.evaluate(input),
            ExprNode::Aggregate(child) => child.evaluate(input),
            ExprNode::Trinary(child) => child.evaluate(input),
            ExprNode::Binary(child) => child.evaluate(input),
            ExprNode::Unary(child) => child.evaluate(input),
            ExprNode::Schedule(child) => child.evaluate(input),
        }
    }
}

impl<'a> From<AnyValue<'a>> for Expr {
    fn from(value: AnyValue<'a>) -> Self {
        Expr::new(ExprNode::Literal(value.into_static()))
    }
}

impl From<SelectExpr> for Expr {
    fn from(selector: SelectExpr) -> Self {
        Expr::new(ExprNode::Selector(selector))
    }
}

impl From<AggExpr> for Expr {
    fn from(agg: AggExpr) -> Self {
        Expr::new(ExprNode::Aggregate(agg))
    }
}

impl From<ScheduleExpr> for Expr {
    fn from(schedule: ScheduleExpr) -> Self {
        Expr::new(ExprNode::Schedule(schedule))
    }
}

impl From<TrinaryExpr> for Expr {
    fn from(trinary: TrinaryExpr) -> Self {
        Expr::new(ExprNode::Trinary(trinary))
    }
}

impl From<BinaryExpr> for Expr {
    fn from(binary: BinaryExpr) -> Self {
        Expr::new(ExprNode::Binary(binary))
    }
}

impl From<UnaryExpr> for Expr {
    fn from(unary: UnaryExpr) -> Self {
        Expr::new(ExprNode::Unary(unary))
    }
}
