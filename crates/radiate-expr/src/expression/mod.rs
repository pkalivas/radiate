mod aggregate;
mod logical;
mod named;
mod ops;
mod projection;
mod schedule;
mod select;

use crate::{
    AnyValue, DataType, Field,
    expression::schedule::{EveryState, ScheduleExpr},
};

use aggregate::{AggExpr, BufferExpr, Rollup};
use logical::When;
pub use named::NamedExpr;
use ops::{BinaryExpr, BinaryOp, TrinaryExpr, TrinaryOp, UnaryExpr, UnaryOp};
pub use projection::*;
use radiate_error::RadiateError;
use radiate_utils::{SmallStr, WindowBuffer};
pub use select::SelectExpr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

mod expr_fields {
    use super::*;
    use crate::DataType;

    pub static STD_DEV: Field = Field::new_const("std_dev", DataType::Float32);
    pub static MEAN: Field = Field::new_const("mean", DataType::Float32);
    pub static MIN: Field = Field::new_const("min", DataType::Float32);
    pub static MAX: Field = Field::new_const("max", DataType::Float32);
    pub static SUM: Field = Field::new_const("sum", DataType::Float32);
    pub static VAR: Field = Field::new_const("var", DataType::Float32);
    pub static SKEW: Field = Field::new_const("skew", DataType::Float32);
    pub static COUNT: Field = Field::new_const("count", DataType::UInt64);
    pub static LAST_VALUE: Field = Field::new_const("last_value", DataType::Float32);
}

pub(crate) type ExprResult<'a> = Result<AnyValue<'a>, RadiateError>;

pub trait ApplyExpr<'a> {
    fn apply(&self, expr: &'a mut Expr) -> AnyValue<'a>;
}

pub trait ExprQuery<I> {
    fn dispatch<'a>(&'a mut self, input: &I) -> ExprResult<'a>;
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(AnyValue<'static>),
    Selector(SelectExpr),
    Aggregate(AggExpr),
    Buffer(BufferExpr),
    Schedule(ScheduleExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Trinary(TrinaryExpr),
}

impl Expr {
    fn try_swap_select_dtype(&mut self, to: DataType) -> bool {
        match self {
            Expr::Selector(SelectExpr::Field(value, field)) => {
                let new_field = field.with_dtype(to);
                *self = Expr::Selector(SelectExpr::Field(value.clone(), new_field));
                true
            }
            _ => false,
        }
    }

    fn try_swap_select_name(&mut self, to: &Field) -> bool {
        match self {
            Expr::Selector(SelectExpr::Field(value, field)) => {
                let new_field = field.with_name(to.name().clone());
                *self = Expr::Selector(SelectExpr::Field(value.clone(), new_field));
                true
            }
            _ => false,
        }
    }

    fn try_swap_select_field_or(mut self, to: &Field, func: impl FnOnce(Self) -> Expr) -> Expr {
        if self.try_swap_select_name(to) {
            return self;
        }

        func(self)
    }

    fn try_swap_agg_rollup_or(mut self, to: Rollup, func: impl FnOnce(Self) -> Expr) -> Expr {
        match self {
            Expr::Aggregate(mut agg) => {
                if agg.rollup != Rollup::Unique {
                    agg.rollup = to;
                    self = Expr::Aggregate(agg);
                    return self;
                }

                func(Expr::Aggregate(agg))
            }
            _ => func(self),
        }
    }

    fn try_reduce_select_agg_rollup_or(
        self,
        field: &Field,
        to: Rollup,
        func: impl FnOnce(Self) -> Expr,
    ) -> Expr {
        self.try_swap_select_field_or(field, |outer| outer.try_swap_agg_rollup_or(to, func))
    }

    pub fn time(mut self) -> Expr {
        self.try_swap_select_dtype(DataType::Duration);
        self
    }

    pub fn value(mut self) -> Expr {
        self.try_swap_select_dtype(DataType::Float32);
        self
    }

    pub fn debug(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Debug))
    }

    pub fn rolling(self, window_size: usize) -> Expr {
        match self {
            Expr::Aggregate(agg) => Expr::Aggregate(AggExpr {
                child: agg.child,
                rollup: agg.rollup,
                buffer: Some(WindowBuffer::with_window(window_size)),
            }),
            Expr::Selector(select) => Expr::Aggregate(AggExpr {
                child: Box::new(Expr::Selector(select)),
                rollup: Rollup::Last,
                buffer: Some(WindowBuffer::with_window(window_size)),
            }),
            _ => Expr::Buffer(BufferExpr::new(self, window_size)),
        }
    }

    /// Aggregates
    pub fn first(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::LAST_VALUE, Rollup::First, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::First))
        })
    }

    pub fn last(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::LAST_VALUE, Rollup::Last, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Last))
        })
    }

    pub fn sum(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::SUM, Rollup::Sum, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Sum))
        })
    }

    pub fn mean(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::MEAN, Rollup::Mean, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Mean))
        })
    }

    pub fn stddev(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::STD_DEV, Rollup::StdDev, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::StdDev))
        })
    }

    pub fn min(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::MIN, Rollup::Min, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Min))
        })
    }

    pub fn max(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::MAX, Rollup::Max, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Max))
        })
    }

    pub fn var(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::VAR, Rollup::Var, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Var))
        })
    }

    pub fn skew(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::SKEW, Rollup::Skew, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Skew))
        })
    }

    pub fn count(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(&expr_fields::COUNT, Rollup::Count, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Count))
        })
    }

    pub fn slope(self) -> Expr {
        self.try_swap_agg_rollup_or(Rollup::Slope, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Slope))
        })
    }

    pub fn unique(self) -> Expr {
        self.try_swap_agg_rollup_or(Rollup::Unique, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Unique))
        })
    }

    pub fn pow(self, exp: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, exp.into(), BinaryOp::Pow))
    }

    /// Comparisons
    pub fn lt(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Lt))
    }

    pub fn lte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Lte))
    }

    pub fn gt(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Gt))
    }

    pub fn gte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Gte))
    }

    pub fn eq(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Eq))
    }

    pub fn ne(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Ne))
    }

    pub fn between(self, low: impl Into<Expr>, high: impl Into<Expr>) -> Expr {
        let low = low.into();
        let high = high.into();

        self.clone().gte(low).and(self.lte(high))
    }

    /// Logic
    pub fn and(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::And))
    }

    pub fn or(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Or))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Not))
    }

    /// Arithmetic
    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Neg))
    }

    pub fn abs(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Abs))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Add))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Sub))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Mul))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn div(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Div))
    }

    pub fn clamp(self, min: impl Into<Expr>, max: impl Into<Expr>) -> Expr {
        Expr::Trinary(TrinaryExpr::new(
            self,
            min.into(),
            max.into(),
            TrinaryOp::Clamp,
        ))
    }

    // scheduling
    pub fn every(self, interval: usize) -> When {
        When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
            interval,
        ))))
    }

    pub fn cast(self, to: DataType) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Cast(to)))
    }
}

impl<I> ExprQuery<I> for Expr
where
    I: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &I) -> ExprResult<'a> {
        match self {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Selector(selector) => selector.dispatch(input),
            Expr::Aggregate(child) => child.dispatch(input),
            Expr::Buffer(child) => child.dispatch(input),
            Expr::Trinary(child) => child.dispatch(input),
            Expr::Binary(child) => child.dispatch(input),
            Expr::Unary(child) => child.dispatch(input),
            Expr::Schedule(child) => child.dispatch(input),
        }
    }
}

impl From<f32> for Expr {
    fn from(value: f32) -> Self {
        Expr::Literal(AnyValue::Float32(value))
    }
}

pub mod expr {
    use super::*;
    use crate::expression::{expr_fields::LAST_VALUE, select::PathBuilder};

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::Literal(value.into())
    }

    pub fn select(name: impl Into<SmallStr>) -> Expr {
        let small_name = name.into();
        Expr::Selector(SelectExpr::Field(
            AnyValue::StrOwned(small_name.clone().into_string()),
            LAST_VALUE.clone(),
        ))
    }

    pub fn select_with_dtype(name: impl Into<SmallStr>, dtype: DataType) -> Expr {
        let small_name = name.into();
        Expr::Selector(SelectExpr::Field(
            AnyValue::StrOwned(small_name.clone().into_string()),
            LAST_VALUE.clone().with_dtype(dtype),
        ))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn path(name: impl Into<AnyValue<'static>>) -> PathBuilder {
        PathBuilder::default().key(name.into())
    }

    pub fn nth(n: usize) -> Expr {
        Expr::Selector(SelectExpr::Nth(n))
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
            interval,
        ))))
    }

    pub fn element() -> Expr {
        Expr::Selector(SelectExpr::Element)
    }
}

impl Add for Expr {
    type Output = Expr;
    fn add(self, rhs: Expr) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs, BinaryOp::Add))
    }
}

impl Sub for Expr {
    type Output = Expr;
    fn sub(self, rhs: Expr) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs, BinaryOp::Sub))
    }
}

impl Mul for Expr {
    type Output = Expr;
    fn mul(self, rhs: Expr) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs, BinaryOp::Mul))
    }
}

impl Div for Expr {
    type Output = Expr;
    fn div(self, rhs: Expr) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs, BinaryOp::Div))
    }
}

impl Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Neg))
    }
}

impl Not for Expr {
    type Output = Expr;
    fn not(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Not))
    }
}
