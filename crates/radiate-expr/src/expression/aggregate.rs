use crate::{AnyValue, DataType, Expr, ExprProjection, ExprQuery, value};
use radiate_utils::{Statistic, WindowBuffer};
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rollup {
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Var,
    Skew,
    Count,
    Unique,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AggExpr {
    pub(super) child: Box<Expr>,
    pub(super) rollup: Rollup,
}

impl AggExpr {
    pub fn new(child: Expr, rollup: Rollup) -> Self {
        Self {
            child: Box::new(child),
            rollup,
        }
    }

    fn compute_rollup<'a>(values: &[AnyValue<'a>], rollup: Rollup) -> AnyValue<'a> {
        let mut stats = Statistic::default();
        let mut dtype = DataType::Null;

        if values.is_empty() {
            return match rollup {
                Rollup::Count => AnyValue::UInt64(0),
                Rollup::Unique => AnyValue::Vector(vec![]),
                _ => AnyValue::Float32(0.0),
            };
        }

        for value in values.iter() {
            if value.is_nested() {
                return AnyValue::Null;
            }

            if dtype == DataType::Null {
                dtype = value.dtype();
            } else if dtype != value.dtype() {
                return AnyValue::Null;
            }

            if let Some(v) = value.clone().extract::<f32>() {
                stats.add(v);
            }
        }

        let result = match rollup {
            Rollup::Mean => AnyValue::Float32(stats.mean()),
            Rollup::StdDev => AnyValue::Float32(stats.std_dev()),
            Rollup::Min => AnyValue::Float32(stats.min()),
            Rollup::Max => AnyValue::Float32(stats.max()),
            Rollup::Sum => AnyValue::Float32(stats.sum()),
            _ => unreachable!(
                "This function should only be called for mean, stddev, min, max, and sum rollups"
            ),
        };

        return result.cast(&dtype).unwrap_or(AnyValue::Null);
    }
}

impl<T> ExprQuery<T> for AggExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        let child_output = self.child.dispatch(input);

        if let Rollup::Unique = self.rollup {
            return value::dedup(child_output).unwrap_or(AnyValue::Null);
        } else if let Rollup::Count = self.rollup {
            return match child_output.len() {
                Some(len) => AnyValue::UInt64(len as u64),
                None => AnyValue::Null,
            };
        }

        match child_output {
            AnyValue::Slice(values) => Self::compute_rollup(values, self.rollup),
            AnyValue::Vector(values) => Self::compute_rollup(&values, self.rollup),
            _ => child_output,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BufferExpr {
    pub(super) buffer: WindowBuffer<AnyValue<'static>>,
    pub(super) child: Box<Expr>,
    pub(super) dtype: DataType,
}

impl BufferExpr {
    pub fn new(child: Expr, window_size: usize) -> Self {
        Self {
            buffer: WindowBuffer::with_window(window_size),
            child: Box::new(child),
            dtype: DataType::Null,
        }
    }
}

impl<T> ExprQuery<T> for BufferExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        let child_output = self.child.dispatch(input).into_static();

        if self.dtype == DataType::Null {
            self.dtype = child_output.dtype();
        } else if self.dtype != child_output.dtype() {
            panic!(
                "BufferExpr received value of type {:?} but expected {:?}",
                child_output.dtype(),
                self.dtype
            );
        }

        self.buffer.push(child_output);
        AnyValue::Slice(&self.buffer.values())
    }
}
