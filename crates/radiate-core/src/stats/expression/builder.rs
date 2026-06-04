use super::{
    Expr, MetricField, MetricKind,
    aggregate::{AggExpr, BufferExpr, Rollup},
    logical::When,
    ops::{BinaryExpr, BinaryOp, TrinaryExpr, TrinaryOp, UnaryExpr, UnaryOp, fuse_affine},
    schedule::{EveryState, ScheduleExpr},
};
use radiate_utils::{AnyValue, DataType, Quantile};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

impl Expr {
    // Rewrites a Selector's kind in-place. Returns true if the rewrite happened.
    fn try_swap_select_kind(&mut self, to: MetricKind) -> bool {
        if let Expr::Selector(sel) = self {
            sel.kind = to;
            return true;
        }
        false
    }

    // Rewrites a Selector's field in-place. Returns true if the rewrite happened.
    fn try_swap_select_field(&mut self, to: MetricField) -> bool {
        if let Expr::Selector(sel) = self {
            sel.field = to;
            return true;
        }
        false
    }

    // If this is a Selector, rewrites its field to `to`; otherwise calls `func`.
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
        field: MetricField,
        to: Rollup,
        func: impl FnOnce(Self) -> Expr,
    ) -> Expr {
        self.try_swap_select_field_or(field, |outer| outer.try_swap_agg_rollup_or(to, func))
    }

    pub fn time(mut self) -> Expr {
        self.try_swap_select_kind(MetricKind::Duration);
        self
    }

    pub fn value(mut self) -> Expr {
        self.try_swap_select_kind(MetricKind::Value);
        self
    }

    pub fn debug(self) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Debug))
    }

    pub fn rolling(self, window_size: usize) -> Expr {
        match self {
            Expr::Aggregate(agg) => {
                Expr::Aggregate(AggExpr::new(*agg.child, agg.rollup).rolling(window_size))
            }
            Expr::Selector(select) => Expr::Aggregate(
                AggExpr::new(Expr::Selector(select), Rollup::Last).rolling(window_size),
            ),
            _ => Expr::Buffer(BufferExpr::new(self, window_size)),
        }
    }

    pub fn first(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::LastValue, Rollup::First, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::First))
        })
    }

    pub fn last(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::LastValue, Rollup::Last, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Last))
        })
    }

    pub fn sum(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Sum, Rollup::Sum, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Sum))
        })
    }

    pub fn mean(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Mean, Rollup::Mean, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Mean))
        })
    }

    pub fn stddev(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::StdDev, Rollup::StdDev, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::StdDev))
        })
    }

    pub fn min(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Min, Rollup::Min, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Min))
        })
    }

    pub fn max(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Max, Rollup::Max, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Max))
        })
    }

    pub fn var(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Var, Rollup::Var, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Var))
        })
    }

    pub fn skew(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Skew, Rollup::Skew, |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Skew))
        })
    }

    pub fn count(self) -> Expr {
        self.try_reduce_select_agg_rollup_or(MetricField::Count, Rollup::Count, |expr| {
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

    /// Returns `self` if it evaluates to a finite number, otherwise `rhs`.
    /// Triggers fallback on Null, NaN, and ±Inf. Short-circuits — `rhs` is only
    /// evaluated when needed, so it's safe to use as a non-trivial default.
    pub fn or_else(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Coalesce))
    }

    /// Elementwise min: `min(self, rhs)`. NaN on one side returns the other.
    /// Use as a ceiling: `expr.min_with(2.0)`.
    pub fn min_with(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Min))
    }

    /// Elementwise max: `max(self, rhs)`. NaN on one side returns the other.
    /// Use as a floor: `expr.max_with(0.05)`.
    pub fn max_with(self, rhs: impl Into<Expr>) -> Expr {
        Expr::Binary(BinaryExpr::new(self, rhs.into(), BinaryOp::Max))
    }

    /// Quantile at `q ∈ [0, 1]` via linear interpolation between adjacent ranks.
    /// On an empty buffer returns 0.0. Filters non-finite samples before sorting.
    /// O(n log n) per evaluation — fine for window sizes ≤ ~1000.
    pub fn quantile(self, q: f32) -> Expr {
        self.try_swap_agg_rollup_or(Rollup::Quantile(q), |expr| {
            Expr::Aggregate(AggExpr::new(expr, Rollup::Quantile(q)))
        })
    }

    pub fn every(self, interval: usize) -> When {
        When::new(Expr::Schedule(ScheduleExpr::Every(EveryState::new(
            interval,
        ))))
    }

    pub fn cast(self, to: DataType) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Cast(to)))
    }

    /// `scale * self + bias`. Replaces `self.mul(lit).add(lit)` with a single
    /// fused Unary(Affine) node. Consecutive affines collapse:
    /// `x.affine(a, b).affine(c, d)` becomes `x.affine(c * a, c * b + d)`.
    pub fn affine(self, scale: f32, bias: f32) -> Expr {
        fuse_affine(self, scale, bias)
    }

    /// Relative error from a target: `(self - target) / target`. Fuses into
    /// a single Affine node. `target == 0` produces a degenerate expression
    /// (division by zero shows up as a NaN/Inf at eval time, then propagates
    /// to the outer Clamp).
    pub fn error_from(self, target: f32) -> Expr {
        // (x - target) / target == x * (1/target) + (-1)
        self.affine(1.0 / target, -1.0)
    }

    /// Streaming P² quantile estimator over the values this expression emits.
    /// Constant memory (5 markers), constant per-eval cost. Approximate but
    /// good for unimodal distributions; sees every observation since
    /// construction or the last `reset()`.
    ///
    /// For an exact quantile over a recent window, use `.rolling(N).quantile(q)`.
    ///
    /// # Panics
    /// `q` must lie in `(0, 1)`.
    pub fn quantile_stream(self, q: f32) -> Expr {
        Expr::Unary(UnaryExpr::new(self, UnaryOp::Quantile(Quantile::new(q))))
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
