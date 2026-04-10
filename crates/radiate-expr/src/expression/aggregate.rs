use radiate_core::Statistic;
use radiate_utils::WindowBuffer;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

use crate::{AnyValue, DataType, Expr, ExprProjection, ExprQuery};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rollup {
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Count,
    Unique,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AggExpr {
    pub(super) child: Arc<Expr>,
    pub(super) rollup: Rollup,
}

impl AggExpr {
    pub fn new(child: Expr, rollup: Rollup) -> Self {
        Self {
            child: Arc::new(child),
            rollup,
        }
    }

    fn compute_rollup<'a>(values: &[AnyValue<'a>], rollup: Rollup) -> AnyValue<'a> {
        let mut stats = Statistic::default();
        let mut dtype = DataType::Null;

        if values.is_empty() {
            return match rollup {
                Rollup::Mean | Rollup::StdDev | Rollup::Min | Rollup::Max | Rollup::Sum => {
                    AnyValue::Float32(0.0)
                }
                Rollup::Count => AnyValue::UInt64(0),
                Rollup::Unique => AnyValue::Vector(vec![]),
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
            Rollup::Count => AnyValue::UInt64(stats.count() as u64),
            Rollup::Unique => AnyValue::Null,
        };

        return result.cast(&dtype).unwrap_or(AnyValue::Null);
    }
}

impl<T> ExprQuery<T> for AggExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        if let Rollup::Unique = self.rollup {
            let child_output = Arc::make_mut(&mut self.child).dispatch(input);
            return match child_output {
                AnyValue::Slice(values) => {
                    let deduped = values.iter().fold(HashSet::new(), |mut acc, v| {
                        acc.insert(v.clone());
                        acc
                    });

                    return AnyValue::Vector(deduped.into_iter().collect());
                }
                AnyValue::Vector(values) => {
                    let deduped = values.into_iter().fold(HashSet::new(), |mut acc, v| {
                        acc.insert(v);
                        acc
                    });

                    AnyValue::Vector(deduped.into_iter().collect())
                }
                _ => AnyValue::Null,
            };
        }
        let child_output = Arc::make_mut(&mut self.child).dispatch(input);

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
    pub(super) child: Arc<Expr>,
}

impl BufferExpr {
    pub fn new(child: Expr, window_size: usize) -> Self {
        Self {
            buffer: WindowBuffer::with_window(window_size),
            child: Arc::new(child),
        }
    }
}

impl<T> ExprQuery<T> for BufferExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> AnyValue<'a> {
        let child_output = Arc::make_mut(&mut self.child).dispatch(input).into_static();
        self.buffer.push(child_output);
        AnyValue::Slice(&self.buffer.values())
    }
}
