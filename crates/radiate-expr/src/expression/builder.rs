use super::{
    Expr,
    aggregate::{AggExpr, BufferExpr, Rollup},
    expr_fields,
    logical::When,
    ops::{BinaryExpr, BinaryOp, TrinaryExpr, TrinaryOp, UnaryExpr, UnaryOp},
    schedule::{EveryState, ScheduleExpr},
    select::SelectExpr,
};
use crate::{AnyValue, DataType, Field};
use radiate_utils::WindowBuffer;
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

impl Expr {
    // Rewrites a Field selector's dtype in-place. Returns true if the rewrite happened.
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

    // Rewrites a Field selector's stat-field name in-place. Returns true if the rewrite happened.
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

    // If this is a Field selector, rewrites its name to `to`; otherwise calls `func`.
    fn try_swap_select_field_or(mut self, to: &Field, func: impl FnOnce(Self) -> Expr) -> Expr {
        if self.try_swap_select_name(to) {
            return self;
        }
        func(self)
    }

    // If this is an Aggregate (non-Unique), rewrites its rollup to `to`; otherwise calls `func`.
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

    // Fuses select("x").agg() into a single Selector node when possible, avoiding a wrapping
    // Aggregate. Falls back to `func` for any other shape.
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

    pub fn every(self, interval: usize) -> When {
        When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
            interval,
        ))))
    }

    pub fn cast(self, to: DataType) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Cast(to)))
    }
}

impl From<f32> for Expr {
    fn from(value: f32) -> Self {
        Expr::Literal(AnyValue::Float32(value))
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
