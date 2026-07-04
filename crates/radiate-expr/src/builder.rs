use crate::nodes::{
    aggregate::{AggExpr, Rollup},
    ops::{BinaryExpr, BinaryOp, TrinaryExpr, TrinaryOp, UnaryExpr, UnaryOp, fuse_affine},
};
use crate::{Expr, MetricField, MetricKind, expr::ExprKind};
use radiate_utils::{DataType, Quantile};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

impl Expr {
    pub fn time(mut self) -> Expr {
        self.try_swap_select_kind(MetricKind::Duration);
        self
    }

    pub fn value(mut self) -> Expr {
        self.try_swap_select_kind(MetricKind::Value);
        self
    }

    pub fn debug(self) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(self, UnaryOp::Debug)))
    }

    pub fn coalesce(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Coalesce,
        )))
    }

    pub fn rolling(self, window_size: usize) -> Expr {
        match self.kind {
            ExprKind::Aggregate(agg) => Expr::new(ExprKind::Aggregate(
                AggExpr::new(*agg.child, agg.rollup).rolling(window_size),
            )),
            ExprKind::Selector(select) => Expr::new(ExprKind::Aggregate(
                AggExpr::new(Expr::new(ExprKind::Selector(select)), Rollup::Last)
                    .rolling(window_size),
            )),
            kind => Expr::new(ExprKind::Aggregate(
                AggExpr::new(Expr::new(kind), Rollup::Last).rolling(window_size),
            )),
        }
    }

    /// Override the minimum number of samples required before the rolling aggregate
    /// emits a value. Defaults to the window size when `.rolling(n)` is called.
    /// Use `.min_samples(1)` to restore partial-window behavior.
    pub fn min_samples(self, n: usize) -> Expr {
        match self.kind {
            ExprKind::Aggregate(agg) => Expr::new(ExprKind::Aggregate(agg.min_samples(n))),
            _ => self,
        }
    }

    pub fn first(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::LastValue, Rollup::First, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::First)))
        })
    }

    pub fn last(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::LastValue, Rollup::Last, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Last)))
        })
    }

    pub fn sum(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Sum, Rollup::Sum, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Sum)))
        })
    }

    pub fn mean(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Mean, Rollup::Mean, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Mean)))
        })
    }

    pub fn stddev(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::StdDev, Rollup::StdDev, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::StdDev)))
        })
    }

    pub fn min(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Min, Rollup::Min, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Min)))
        })
    }

    pub fn max(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Max, Rollup::Max, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Max)))
        })
    }

    pub fn var(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Var, Rollup::Var, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Var)))
        })
    }

    pub fn skew(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Skew, Rollup::Skew, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Skew)))
        })
    }

    pub fn count(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Count, Rollup::Count, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Count)))
        })
    }

    pub fn slope(self) -> Expr {
        self.try_swap_agg_rollup_or(Rollup::Slope, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Slope)))
        })
    }

    pub fn unique(self) -> Expr {
        self.try_swap_agg_rollup_or(Rollup::Unique, |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(expr, Rollup::Unique)))
        })
    }

    pub fn pow(self, exp: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            exp.into(),
            BinaryOp::Pow,
        )))
    }

    pub fn lt(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Lt,
        )))
    }

    pub fn lte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Lte,
        )))
    }

    pub fn gt(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Gt,
        )))
    }

    pub fn gte(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Gte,
        )))
    }

    pub fn eq(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Eq,
        )))
    }

    pub fn ne(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Ne,
        )))
    }

    pub fn between(self, low: impl Into<Expr>, high: impl Into<Expr>) -> Expr {
        let low = low.into();
        let high = high.into();
        self.clone().gte(low).and(self.lte(high))
    }

    pub fn and(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::And,
        )))
    }

    pub fn or(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Or,
        )))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(self, UnaryOp::Not)))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(self, UnaryOp::Neg)))
    }

    pub fn abs(self) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(self, UnaryOp::Abs)))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Add,
        )))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Sub,
        )))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Mul,
        )))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn div(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Div,
        )))
    }

    pub fn clamp(self, min: impl Into<Expr>, max: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Trinary(TrinaryExpr::new(
            self,
            min.into(),
            max.into(),
            TrinaryOp::Clamp,
        )))
    }

    /// Returns `self` if it evaluates to a finite number, otherwise `rhs`.
    /// Triggers fallback on Null, NaN, and ±Inf. Short-circuits — `rhs` is only
    /// evaluated when needed, so it's safe to use as a non-trivial default.
    pub fn or_else(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Coalesce,
        )))
    }

    /// Elementwise min: `min(self, rhs)`. NaN on one side returns the other.
    /// Use as a ceiling: `expr.min_with(2.0)`.
    pub fn min_with(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Min,
        )))
    }

    /// Elementwise max: `max(self, rhs)`. NaN on one side returns the other.
    /// Use as a floor: `expr.max_with(0.05)`.
    pub fn max_with(self, rhs: impl Into<Expr>) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Max,
        )))
    }

    /// Quantile at `q ∈ [0, 1]` via linear interpolation between adjacent ranks.
    /// On an empty buffer returns 0.0. Filters non-finite samples before sorting.
    /// O(n log n) per evaluation — fine for window sizes ≤ ~1000.
    pub fn quantile(self, q: f32) -> Expr {
        self.try_swap_agg_rollup_or(Rollup::Quantile(Quantile::new(q)), |expr| {
            Expr::new(ExprKind::Aggregate(AggExpr::new(
                expr,
                Rollup::Quantile(Quantile::new(q)),
            )))
        })
    }

    pub fn stagnation(self, epsilon: f32) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(
            self,
            UnaryOp::Stagnation {
                epsilon,
                last_value: None,
                count: 0,
            },
        )))
    }

    pub fn cast(self, to: DataType) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(self, UnaryOp::Cast(to))))
    }

    /// Relative error from a target: `(self - target) / target`. Fuses into
    /// a single Affine node. `target == 0` produces a degenerate expression
    /// (division by zero shows up as a NaN/Inf at eval time, then propagates
    /// to the outer Clamp).
    pub fn error(self, target: f32) -> Expr {
        // (x - target) / target == x * (1/target) + (-1)
        fuse_affine(self, 1.0 / target, -1.0)
    }

    fn try_swap_select_kind(&mut self, to: MetricKind) -> bool {
        if let ExprKind::Selector(sel) = &mut self.kind {
            sel.kind = to;
            return true;
        }
        false
    }

    fn try_swap_select_field(&mut self, to: MetricField) -> bool {
        if let ExprKind::Selector(sel) = &mut self.kind {
            sel.field = to;
            return true;
        }
        false
    }

    fn try_swap_select_field_or(
        mut self,
        to: MetricField,
        func: impl FnOnce(Self) -> Expr,
    ) -> Expr {
        if self.try_swap_select_field(to) {
            return self;
        }
        func(self)
    }

    fn try_swap_agg_rollup_or(mut self, to: Rollup, func: impl FnOnce(Self) -> Expr) -> Expr {
        if let ExprKind::Aggregate(ref mut agg) = self.kind {
            if agg.rollup != Rollup::Unique {
                agg.rollup = to;
                return self;
            }
        }
        func(self)
    }

    fn try_reduce_select_agg_rollup_or(
        self,
        field: MetricField,
        to: Rollup,
        func: impl FnOnce(Self) -> Expr,
    ) -> Expr {
        self.try_swap_select_field_or(field, |outer| outer.try_swap_agg_rollup_or(to, func))
    }
}

macro_rules! impl_from_literal {
    ($($ty:ty => $variant:ident),*) => {
        $(
            impl From<$ty> for Expr {
                fn from(value: $ty) -> Self {
                    use crate::expr::ExprKind;
                    Expr::new(ExprKind::Literal(value.into()))
                }
            }
        )*
    };
}

impl_from_literal!(
    u8 => UInt8,
    u16 => UInt16,
    u32 => UInt32,
    u64 => UInt64,
    u128 => UInt128,

    i8 => Int8,
    i16 => Int16,
    i32 => Int32,
    i64 => Int64,
    i128 => Int128,

    f32 => Float32,
    f64 => Float64,

    bool => Bool,
    char => Char,
    String => Str,

    usize => Usize
);

impl<T> Add<T> for Expr
where
    T: Into<Expr>,
{
    type Output = Expr;
    fn add(self, rhs: T) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Add,
        )))
    }
}

impl<T> Sub<T> for Expr
where
    T: Into<Expr>,
{
    type Output = Expr;
    fn sub(self, rhs: T) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Sub,
        )))
    }
}

impl<T> Mul<T> for Expr
where
    T: Into<Expr>,
{
    type Output = Expr;
    fn mul(self, rhs: T) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Mul,
        )))
    }
}

impl<T> Div<T> for Expr
where
    T: Into<Expr>,
{
    type Output = Expr;
    fn div(self, rhs: T) -> Expr {
        Expr::new(ExprKind::Binary(BinaryExpr::new(
            self,
            rhs.into(),
            BinaryOp::Div,
        )))
    }
}

impl Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(self, UnaryOp::Neg)))
    }
}

impl Not for Expr {
    type Output = Expr;
    fn not(self) -> Expr {
        Expr::new(ExprKind::Unary(UnaryExpr::new(self, UnaryOp::Not)))
    }
}
