use crate::{ExprEval, ExprResult, ExprSelect, SelectExpr};
use crate::{
    Selector, metric_fields,
    nodes::{AggExpr, BinaryExpr, IndexState, ScheduleExpr, TrinaryExpr, UnaryExpr, When},
};
use radiate_error::RadiateError;
use radiate_utils::{AnyValue, SmallStr};
use radiate_utils::{DataType, sentry_id};
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
            name: SmallStr::from_string(format!("Expr<{:?}>.{:?}", id, node)),
            id,
            node,
        }
    }

    pub fn select<'a, O>(&'a mut self, val: impl ExprSelect) -> Result<O, RadiateError>
    where
        O: TryFrom<AnyValue<'a>, Error = RadiateError>,
    {
        let result = self.eval(&val)?;
        O::try_from(result)
    }

    pub fn identity() -> Expr {
        Expr::new(ExprNode::Selector(SelectExpr::new(Selector::Identity)))
    }

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::new(ExprNode::Literal(value.into()))
    }

    pub fn range(sel: impl Into<std::ops::Range<usize>>) -> Expr {
        Expr::new(ExprNode::Selector(SelectExpr::new(sel.into())))
    }

    pub fn metric(name: impl Into<SmallStr>) -> Expr {
        let name = name.into();
        Expr::new(ExprNode::Selector(SelectExpr::new(Selector::Metric {
            name,
            field: metric_fields::LAST_VALUE,
            dtype: DataType::Float32,
        })))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::new(ExprNode::Schedule(ScheduleExpr::Interval(
            IndexState::new(interval),
        ))))
    }

    pub fn throttle(duration: std::time::Duration) -> When {
        When::new(Expr::new(ExprNode::Schedule(ScheduleExpr::Duration(
            duration.into(),
        ))))
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

impl<'a, T> ExprEval<'a, T> for Expr
where
    T: ExprSelect,
{
    #[inline]
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        match &mut self.node {
            ExprNode::Literal(value) => Ok(value.clone()),
            ExprNode::Selector(selector) => selector.eval(metrics),
            ExprNode::Aggregate(child) => child.eval(metrics),
            ExprNode::Trinary(child) => child.eval(metrics),
            ExprNode::Binary(child) => child.eval(metrics),
            ExprNode::Unary(child) => child.eval(metrics),
            ExprNode::Schedule(child) => child.eval(metrics),
        }
    }
}
