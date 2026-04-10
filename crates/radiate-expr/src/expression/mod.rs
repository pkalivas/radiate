mod aggregate;
mod logical;
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
use ops::{BinaryExpr, BinaryOp, TrinaryExpr, TrinaryOp, UnaryExpr, UnaryOp};
pub use projection::*;
use radiate_utils::SmallStr;
pub use select::SelectExpr;
use std::fmt::Debug;

mod expr_fields {
    use super::*;
    use crate::DataType;

    pub static STD_DEV: Field = Field::new_const("std_dev", DataType::Float32);
    pub static MEAN: Field = Field::new_const("mean", DataType::Float32);
    pub static MIN: Field = Field::new_const("min", DataType::Float32);
    pub static MAX: Field = Field::new_const("max", DataType::Float32);
    pub static SUM: Field = Field::new_const("sum", DataType::Float32);
    pub static COUNT: Field = Field::new_const("count", DataType::UInt64);
    pub static LAST_VALUE: Field = Field::new_const("last_value", DataType::Float32);
    // pub static VERSION: Field = Field::new_const("version", DataType::UInt64);
    // pub static UPDATE_COUNT: Field = Field::new_const("update_count", DataType::UInt64);
}

pub trait ApplyExpr<'a> {
    fn apply(&self, expr: &'a mut Expr) -> AnyValue<'a>;
}

pub trait ExprQuery<I> {
    fn dispatch<'a>(&'a mut self, input: &I) -> AnyValue<'a>;
}

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

    pub fn time(mut self) -> Expr {
        self.try_swap_select_dtype(DataType::Duration);
        self
    }

    pub fn value(mut self) -> Expr {
        self.try_swap_select_dtype(DataType::Float32);
        self
    }

    pub fn rolling(self, window_size: usize) -> Expr {
        Expr::Buffer(BufferExpr::new(self, window_size))
    }

    /// Aggregates
    pub fn sum(self) -> Expr {
        self.try_swap_select_field_or(&expr_fields::SUM, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Sum))
        })
    }

    pub fn mean(self) -> Expr {
        self.try_swap_select_field_or(&expr_fields::MEAN, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Mean))
        })
    }

    pub fn stddev(self) -> Expr {
        self.try_swap_select_field_or(&expr_fields::STD_DEV, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::StdDev))
        })
    }

    pub fn min(self) -> Expr {
        self.try_swap_select_field_or(&expr_fields::MIN, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Min))
        })
    }

    pub fn max(self) -> Expr {
        self.try_swap_select_field_or(&expr_fields::MAX, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Max))
        })
    }

    pub fn count(self) -> Expr {
        self.try_swap_select_field_or(&expr_fields::COUNT, |s| {
            Expr::Aggregate(AggExpr::new(s, Rollup::Count))
        })
    }

    pub fn unique(self) -> Expr {
        Expr::Aggregate(AggExpr::new(self, Rollup::Unique))
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

    pub fn not(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Not))
    }

    /// Arithmetic
    pub fn neg(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Neg))
    }

    pub fn abs(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Abs))
    }

    pub fn add(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Add))
    }

    pub fn sub(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Sub))
    }

    pub fn mul(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Mul))
    }

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
}

impl<I> ExprQuery<I> for Expr
where
    I: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &I) -> AnyValue<'a> {
        match self {
            Expr::Literal(value) => value.clone(),
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
}
