use crate::nodes::{AggExpr, BinaryExpr, EveryState, ScheduleExpr, TrinaryExpr, UnaryExpr, When};
use crate::{Evaluate, ExprResult, ExprSelector, MetricField, MetricKind, SelectExpr};
use radiate_utils::sentry_id;
use radiate_utils::{AnyValue, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;

sentry_id!(ExprId);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
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
    pub(crate) kind: ExprKind,
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        let id = ExprId::new();
        Self {
            name: SmallStr::from_string(format!("Expr<{:?}>.{:?}", id, kind)),
            id,
            kind,
        }
    }

    pub fn identity() -> Expr {
        Expr::new(ExprKind::Selector(SelectExpr {
            metric: None,
            field: MetricField::LastValue,
            kind: MetricKind::Value,
        }))
    }

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::new(ExprKind::Literal(value.into()))
    }

    pub fn select(name: impl Into<SmallStr>) -> Expr {
        Expr::new(ExprKind::Selector(SelectExpr::new(name)))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::new(ExprKind::Schedule(ScheduleExpr::Every(
            EveryState::new(interval),
        ))))
    }

    pub fn kind(&self) -> &ExprKind {
        &self.kind
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

    pub fn try_extract_lit<T: TryFrom<AnyValue<'static>>>(&self) -> Option<T> {
        if let ExprKind::Literal(value) = &self.kind {
            T::try_from(value.clone()).ok()
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        match &mut self.kind {
            ExprKind::Literal(_) | ExprKind::Selector(_) => {}
            ExprKind::Aggregate(a) => a.reset(),
            ExprKind::Schedule(ScheduleExpr::Every(s)) => s.reset(),
            ExprKind::Binary(b) => {
                b.lhs.reset();
                b.rhs.reset();
            }
            ExprKind::Unary(u) => u.reset(),
            ExprKind::Trinary(t) => {
                t.first.reset();
                t.second.reset();
                t.third.reset();
            }
        }
    }
}

impl<'a, T> Evaluate<'a, T> for Expr
where
    T: ExprSelector,
{
    #[inline]
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        match &mut self.kind {
            ExprKind::Literal(value) => Ok(value.clone()),
            ExprKind::Selector(selector) => selector.eval(metrics),
            ExprKind::Aggregate(child) => child.eval(metrics),
            ExprKind::Trinary(child) => child.eval(metrics),
            ExprKind::Binary(child) => child.eval(metrics),
            ExprKind::Unary(child) => child.eval(metrics),
            ExprKind::Schedule(child) => child.eval(metrics),
        }
    }
}
