mod aggregate;
mod builder;
mod logical;
mod named;
mod ops;
mod projection;
mod schedule;
mod select;
mod traits;

pub use named::NamedExpr;
pub use projection::*;
pub use select::SelectExpr;
pub(crate) use traits::ExprResult;
pub use traits::{ApplyExpr, ExprQuery};

use crate::{AnyValue, DataType};
use aggregate::{AggExpr, BufferExpr};
use logical::When;
use ops::{BinaryExpr, TrinaryExpr, UnaryExpr};
use radiate_utils::SmallStr;
use schedule::{EveryState, ScheduleExpr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod expr_fields {
    use crate::{DataType, Field};

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
