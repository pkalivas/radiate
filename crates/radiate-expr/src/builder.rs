use crate::{
    Expr, ExprNode,
    ops::{RollupOp, ScheduleOp},
};
use crate::{
    When,
    ops::{BinaryOp, SelectOp, TrinaryOp, UnaryOp},
};
use radiate_utils::{AnyValue, DataType, SmallStr, WindowBuffer};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

impl Expr {
    pub fn identity() -> Expr {
        Expr::from(SelectOp::Identity)
    }

    pub fn lit(value: impl Into<AnyValue<'static>>) -> Expr {
        Expr::from(value.into())
    }

    pub fn range(sel: impl Into<std::ops::Range<usize>>) -> Expr {
        let range = sel.into();
        Expr::from(SelectOp::Range(range.start, range.end))
    }

    pub fn select(name: impl Into<SmallStr>) -> Expr {
        Expr::from(SelectOp::Field(name.into()))
    }

    pub fn warmup(period: usize) -> When {
        When::new(Expr::new(ExprNode::Schedule(ScheduleOp::Warmup {
            period,
            current: 0,
        })))
    }

    pub fn when(cond: impl Into<Expr>) -> When {
        When::new(cond.into())
    }

    pub fn every(interval: usize) -> When {
        When::new(Expr::new(ExprNode::Schedule(ScheduleOp::Interval {
            count: 0,
            limit: interval,
        })))
    }

    pub fn throttle(duration: std::time::Duration) -> When {
        When::new(Expr::new(ExprNode::Schedule(ScheduleOp::Duration {
            last: None,
            interval: duration,
        })))
    }

    pub fn time(self) -> Expr {
        self.cast(DataType::Duration)
    }

    pub fn value(self) -> Expr {
        self.cast(DataType::Float32)
    }

    pub fn debug(self) -> Expr {
        self.unary(UnaryOp::Debug)
    }

    pub fn attr(self, attr: impl Into<SmallStr>) -> Expr {
        match self.node {
            ExprNode::Selector(selector) => Expr::from(SelectOp::Nested {
                parent: Box::new(selector),
                child: Box::new(SelectOp::Field(attr.into())),
            }),
            _ => self,
        }
    }

    pub fn rolling(self, window_size: usize) -> Expr {
        Expr::new(ExprNode::Rolling {
            child: Box::new(self),
            buffer: WindowBuffer::with_capacity(window_size),
        })
    }

    pub fn coalesce(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Coalesce)
    }

    pub fn first(self) -> Expr {
        self.reducer(RollupOp::First)
    }

    pub fn last(self) -> Expr {
        self.reducer(RollupOp::Last)
    }

    pub fn sum(self) -> Expr {
        self.reducer(RollupOp::Sum)
    }

    pub fn mean(self) -> Expr {
        self.reducer(RollupOp::Mean)
    }

    pub fn stddev(self) -> Expr {
        self.reducer(RollupOp::StdDev)
    }

    pub fn min(self) -> Expr {
        self.reducer(RollupOp::Min)
    }

    pub fn max(self) -> Expr {
        self.reducer(RollupOp::Max)
    }

    pub fn var(self) -> Expr {
        self.reducer(RollupOp::Var)
    }

    pub fn skew(self) -> Expr {
        self.reducer(RollupOp::Skew)
    }

    pub fn count(self) -> Expr {
        self.reducer(RollupOp::Count)
    }

    pub fn slope(self) -> Expr {
        self.reducer(RollupOp::Slope)
    }

    pub fn unique(self) -> Expr {
        self.reducer(RollupOp::Unique)
    }

    pub fn pow(self, exp: impl Into<Expr>) -> Expr {
        self.binary(exp.into(), BinaryOp::Pow)
    }

    pub fn lt(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Lt)
    }

    pub fn lte(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Lte)
    }

    pub fn gt(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Gt)
    }

    pub fn gte(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Gte)
    }

    pub fn eq(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Eq)
    }

    pub fn ne(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Ne)
    }

    pub fn between(self, low: impl Into<Expr>, high: impl Into<Expr>) -> Expr {
        let low = low.into();
        let high = high.into();
        self.clone().gte(low).and(self.lte(high))
    }

    pub fn and(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::And)
    }

    pub fn or(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Or)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Expr {
        self.unary(UnaryOp::Not)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Expr {
        self.unary(UnaryOp::Neg)
    }

    pub fn abs(self) -> Expr {
        self.unary(UnaryOp::Abs)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Add)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Sub)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Mul)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn div(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Div)
    }

    pub fn clamp(self, min: impl Into<Expr>, max: impl Into<Expr>) -> Expr {
        self.trinary(min.into(), max.into(), TrinaryOp::Clamp)
    }

    pub fn or_else(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Coalesce)
    }

    pub fn min_with(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Min)
    }
    pub fn max_with(self, rhs: impl Into<Expr>) -> Expr {
        self.binary(rhs.into(), BinaryOp::Max)
    }

    pub fn quantile(self, q: f32) -> Expr {
        self.reducer(RollupOp::Quantile(q))
    }

    pub fn stagnation(self, epsilon: f32) -> Expr {
        self.unary(UnaryOp::Stagnation {
            epsilon,
            last_value: None,
            count: 0,
        })
    }

    pub fn cast(self, to: DataType) -> Expr {
        self.unary(UnaryOp::Cast(to))
    }

    /// Relative error from a target: `(self - target) / target`. Fuses into
    /// a single Affine node. `target == 0` produces a degenerate expression
    /// (division by zero shows up as a NaN/Inf at eval time, then propagates
    /// to the outer Clamp).
    pub fn error(self, target: f32) -> Expr {
        // (x - target) / target == x * (1/target) + (-1)
        self.binary(Expr::from(1.0 / target), BinaryOp::Mul)
            .add(Expr::from(-1.0))
            .compile()
    }

    fn unary(self, op: UnaryOp) -> Expr {
        Expr::new(ExprNode::Unary {
            child: Box::new(self),
            op,
        })
    }

    fn binary(self, rhs: Expr, op: BinaryOp) -> Expr {
        Expr::new(ExprNode::Binary {
            lhs: Box::new(self),
            rhs: Box::new(rhs),
            op,
        })
    }

    fn trinary(self, second: Expr, third: Expr, op: TrinaryOp) -> Expr {
        Expr::new(ExprNode::Trinary {
            first: Box::new(self),
            second: Box::new(second),
            third: Box::new(third),
            op,
        })
    }

    fn reducer(self, rollup: RollupOp) -> Expr {
        Expr::new(ExprNode::Reduce {
            child: Box::new(self),
            rollup,
        })
    }
}

macro_rules! impl_from_literal {
    ($($ty:ty => $variant:ident),*) => {
        $(
            impl From<$ty> for Expr {
                fn from(value: $ty) -> Self {
                    use crate::ExprNode;
                    Expr::new(ExprNode::Literal(value.into()))
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
        Expr::new(ExprNode::Binary {
            lhs: Box::new(self),
            rhs: Box::new(rhs.into()),
            op: BinaryOp::Add,
        })
    }
}

impl<T> Sub<T> for Expr
where
    T: Into<Expr>,
{
    type Output = Expr;
    fn sub(self, rhs: T) -> Expr {
        Expr::new(ExprNode::Binary {
            lhs: Box::new(self),
            rhs: Box::new(rhs.into()),
            op: BinaryOp::Sub,
        })
    }
}

impl<T> Mul<T> for Expr
where
    T: Into<Expr>,
{
    type Output = Expr;
    fn mul(self, rhs: T) -> Expr {
        Expr::new(ExprNode::Binary {
            lhs: Box::new(self),
            rhs: Box::new(rhs.into()),
            op: BinaryOp::Mul,
        })
    }
}

impl<T> Div<T> for Expr
where
    T: Into<Expr>,
{
    type Output = Expr;
    fn div(self, rhs: T) -> Expr {
        Expr::new(ExprNode::Binary {
            lhs: Box::new(self),
            rhs: Box::new(rhs.into()),
            op: BinaryOp::Div,
        })
    }
}

impl Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        Expr::new(ExprNode::Unary {
            child: Box::new(self),
            op: UnaryOp::Neg,
        })
    }
}

impl Not for Expr {
    type Output = Expr;
    fn not(self) -> Expr {
        Expr::new(ExprNode::Unary {
            child: Box::new(self),
            op: UnaryOp::Not,
        })
    }
}
