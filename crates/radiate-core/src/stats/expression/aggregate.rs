use super::{Evaluate, Expr, ExprResult};
use crate::MetricSet;
use radiate_error::radiate_bail;
use radiate_utils::{AnyValue, DataType, Quantile, Slope, Statistic, WindowBuffer, dedup_slice};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rollup {
    First,
    Last,
    Mean,
    StdDev,
    Min,
    Max,
    Sum,
    Var,
    Skew,
    Count,
    Unique,
    Slope,
    Quantile(f32),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AggExpr {
    pub(super) child: Box<Expr>,
    pub(super) rollup: Rollup,
    pub(super) buffer: Option<WindowBuffer<AnyValue<'static>>>,
}

impl AggExpr {
    pub fn new(child: Expr, rollup: Rollup) -> Self {
        Self {
            child: Box::new(child),
            rollup,
            buffer: None,
        }
    }

    pub fn rolling(mut self, window_size: usize) -> Self {
        self.buffer = Some(WindowBuffer::with_capacity(window_size));
        self
    }

    pub(super) fn reset(&mut self) {
        if let Some(buf) = &mut self.buffer {
            buf.clear();
        }
        self.child.reset();
    }

    fn compute_rollup<'a>(
        values: &[AnyValue<'a>],
        rollup: Rollup,
        dtype: DataType,
    ) -> ExprResult<'a> {
        if values.is_empty() {
            return match rollup {
                Rollup::Count => Ok(AnyValue::UInt64(0)),
                _ => Ok(AnyValue::Float32(0.0)),
            };
        }

        if values.len() == 1 {
            return match rollup {
                Rollup::Count => Ok(AnyValue::UInt64(1)),
                Rollup::Unique => Ok(values[0].clone()),
                _ => Ok(values[0].clone()),
            };
        }

        if let Rollup::Unique = rollup {
            return Ok(dedup_slice(values));
        } else if let Rollup::Count = rollup {
            return Ok(AnyValue::UInt64(values.len() as u64));
        } else if let Rollup::First = rollup {
            return Ok(values[0].clone());
        } else if let Rollup::Last = rollup {
            return Ok(values[values.len() - 1].clone());
        } else if let Rollup::Slope = rollup {
            if values.len() < 2 {
                return Ok(AnyValue::Float32(0.0));
            }

            let slope = values
                .iter()
                .filter_map(|v| v.extract::<f32>())
                .collect::<Slope<f32>>();

            return Ok(AnyValue::Float32(slope.value().unwrap_or(0.0)));
        } else if let Rollup::Quantile(q) = rollup {
            let mut quantile = Quantile::new(q);
            for v in values.iter().filter_map(|v| v.extract::<f32>()) {
                if v.is_finite() {
                    quantile.add(v);
                }
            }

            return Ok(quantile
                .value()
                .map(AnyValue::Float32)
                .unwrap_or(AnyValue::Null));
        }

        let stats = values
            .iter()
            .filter_map(|val| val.extract::<f32>())
            .collect::<Statistic>();

        let result = match rollup {
            Rollup::Mean => AnyValue::Float32(stats.mean()),
            Rollup::StdDev => AnyValue::Float32(stats.std_dev().unwrap()),
            Rollup::Min => AnyValue::Float32(stats.min()),
            Rollup::Max => AnyValue::Float32(stats.max()),
            Rollup::Sum => AnyValue::Float32(stats.sum()),
            Rollup::Count => AnyValue::UInt64(stats.count() as u64),
            _ => AnyValue::Null,
        };

        Ok(result.cast(&dtype).unwrap_or(AnyValue::Null))
    }
}

impl Evaluate for AggExpr {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
        let child_output = self.child.eval(metrics)?;
        let dtype = child_output.dtype();

        if let Some(buffer) = &mut self.buffer {
            buffer.push(child_output.into_static());
            return Self::compute_rollup(buffer.values(), self.rollup, dtype);
        }

        match child_output {
            AnyValue::Slice(values) => {
                let elem_dtype = if let DataType::List(inner) = dtype {
                    *inner
                } else {
                    dtype
                };
                Self::compute_rollup(values, self.rollup, elem_dtype)
            }
            AnyValue::Vector(values) => {
                let elem_dtype = if let DataType::List(inner) = dtype {
                    *inner
                } else {
                    dtype
                };
                Self::compute_rollup(&values, self.rollup, elem_dtype)
            }
            _ => match self.rollup {
                Rollup::Count => Ok(AnyValue::UInt64(1)),
                Rollup::Unique => Ok(AnyValue::Vector(vec![child_output])),
                _ => Ok(child_output),
            },
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BufferExpr {
    pub(super) buffer: WindowBuffer<AnyValue<'static>>,
    pub(super) child: Box<Expr>,
    pub(super) dtype: DataType,
}

impl BufferExpr {
    pub fn new(child: Expr, window_size: usize) -> Self {
        Self {
            buffer: WindowBuffer::with_capacity(window_size),
            child: Box::new(child),
            dtype: DataType::Null,
        }
    }

    pub(super) fn reset(&mut self) {
        self.buffer.clear();
        self.dtype = DataType::Null;
        self.child.reset();
    }
}

impl Evaluate for BufferExpr {
    fn eval<'a>(&'a mut self, metrics: &MetricSet) -> ExprResult<'a> {
        let child_output = self.child.eval(metrics)?.into_static();

        if child_output.is_nested() {
            radiate_bail!(Expr: "BufferExpr does not support nested values");
        }

        if self.dtype == DataType::Null {
            self.dtype = child_output.dtype();
        } else if self.dtype != child_output.dtype() {
            radiate_bail!(Expr:
                "BufferExpr received value of type {:?} but expected {:?}",
                child_output.dtype(),
                self.dtype
            );
        }

        self.buffer.push(child_output);
        Ok(AnyValue::Slice(self.buffer.values()))
    }
}
