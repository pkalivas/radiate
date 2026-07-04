mod named;
pub mod nodes;
mod select;
mod set;
mod traits;

pub use named::NamedExpr;
pub use select::{MetricField, MetricKind, SelectExpr};
pub use set::ExprSet;
pub(crate) use traits::ExprResult;
pub use traits::{Evaluate, ExprSelector};

use nodes::{AggExpr, BinaryExpr, EveryState, ScheduleExpr, TrinaryExpr, UnaryExpr, When};
use radiate_utils::{AnyValue, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(AnyValue<'static>),
    Selector(SelectExpr),
    Aggregate(AggExpr),
    Schedule(ScheduleExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Trinary(TrinaryExpr),
}

impl Expr {
    /// Recursively clears state in stateful operators: rolling-window buffers
    /// in `Aggregate`/`Buffer` nodes and counters in `Schedule::Every`. Children
    /// of binary/unary/trinary nodes are also visited. Leaf nodes (literals,
    /// selectors) are unaffected.
    ///
    /// Use after an engine restart or whenever the controller should "forget"
    /// accumulated history.
    pub fn reset(&mut self) {
        match self {
            Expr::Literal(_) | Expr::Selector(_) => {}
            Expr::Aggregate(a) => a.reset(),
            Expr::Schedule(ScheduleExpr::Every(s)) => s.reset(),
            Expr::Binary(b) => {
                b.lhs.reset();
                b.rhs.reset();
            }
            Expr::Unary(u) => {
                u.reset();
            }
            Expr::Trinary(t) => {
                t.first.reset();
                t.second.reset();
                t.third.reset();
            }
        }
    }

    pub fn alias(self, name: impl Into<SmallStr>) -> NamedExpr {
        NamedExpr::new(name, self)
    }

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::Literal(value.into())
    }

    pub fn select(name: impl Into<SmallStr>) -> Expr {
        Expr::Selector(SelectExpr::new(name))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
            interval,
        ))))
    }

    pub fn identity() -> Expr {
        Expr::Selector(SelectExpr {
            metric: None,
            field: MetricField::LastValue,
            kind: MetricKind::Value,
        })
    }
}

impl<T> Evaluate<T> for Expr
where
    T: ExprSelector,
{
    fn eval<'a>(&'a mut self, metrics: &T) -> ExprResult<'a> {
        match self {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Selector(selector) => selector.eval(metrics),
            Expr::Aggregate(child) => child.eval(metrics),
            Expr::Trinary(child) => child.eval(metrics),
            Expr::Binary(child) => child.eval(metrics),
            Expr::Unary(child) => child.eval(metrics),
            Expr::Schedule(child) => child.eval(metrics),
        }
    }
}
