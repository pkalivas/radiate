use crate::{EvalExpr, ExprResult, ExprSelect, nodes::Selector, query::traits::NoInput};
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

    pub fn tick(&mut self) -> ExprResult<'static> {
        self.evaluate(&NoInput).map(|v| v.into_static())
    }

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

    pub fn throttle(duration: std::time::Duration) -> Expr {
        Expr::from(ScheduleExpr::Duration(duration.into()))
    }

    pub fn kind(&self) -> &ExprNode {
        &self.node
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

impl<'a, T> EvalExpr<'a, T> for Expr
where
    T: ExprSelect<'a>,
{
    #[inline]
    fn evaluate(&'a mut self, metrics: &'a T) -> ExprResult<'a> {
        match &mut self.node {
            ExprNode::Literal(value) => Ok(value.clone()),
            ExprNode::Selector(selector) => selector.evaluate(metrics),
            ExprNode::Aggregate(child) => child.evaluate(metrics),
            ExprNode::Trinary(child) => child.evaluate(metrics),
            ExprNode::Binary(child) => child.evaluate(metrics),
            ExprNode::Unary(child) => child.evaluate(metrics),
            ExprNode::Schedule(child) => child.evaluate(metrics),
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
