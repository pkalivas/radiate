use crate::{Evaluate, Expr, ExprResult, ExprSelector};
use radiate_error::radiate_bail;
use radiate_utils::{AnyValue, DataType, Quantile, Slope, Statistic, WindowBuffer, dedup_slice};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
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
    Quantile(Quantile<f32>),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AggExpr {
    pub(crate) child: Box<Expr>,
    pub(crate) rollup: Rollup,
    pub(crate) buffer: Option<WindowBuffer<AnyValue<'static>>>,
    pub(crate) min_samples: usize,
}

impl AggExpr {
    pub fn new(child: Expr, rollup: Rollup) -> Self {
        Self {
            child: Box::new(child),
            rollup,
            buffer: None,
            min_samples: 0,
        }
    }

    pub fn rolling(mut self, window_size: usize) -> Self {
        self.buffer = Some(WindowBuffer::with_capacity(window_size));
        self.min_samples = window_size;
        self
    }

    pub fn min_samples(mut self, n: usize) -> Self {
        self.min_samples = n;
        self
    }

    pub(crate) fn reset(&mut self) {
        if let Some(buf) = &mut self.buffer {
            buf.clear();
        }
        self.child.reset();

        if let Rollup::Quantile(q) = &mut self.rollup {
            q.clear();
        }
    }

    fn compute_rollup<'a>(
        values: &[AnyValue<'a>],
        rollup: &mut Rollup,
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
        } else if let Rollup::Quantile(quantile) = rollup {
            quantile.clear();
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

impl<'a, T> Evaluate<'a, T> for AggExpr
where
    T: ExprSelector,
{
    fn eval(&'a mut self, metrics: &T) -> ExprResult<'a> {
        let child_output = self.child.eval(metrics)?;
        let dtype = child_output.dtype();

        if let Some(buffer) = &mut self.buffer {
            if child_output.is_nested() {
                radiate_bail!(Expr: "AggExpr with rolling window does not support nested values");
            }

            buffer.push(child_output.into_static());

            if buffer.values().len() < self.min_samples {
                return Ok(AnyValue::Null);
            }

            return Self::compute_rollup(buffer.values(), &mut self.rollup, dtype);
        }

        match child_output {
            AnyValue::Slice(values) => {
                let elem_dtype = if let DataType::List(inner) = dtype {
                    *inner
                } else {
                    dtype
                };
                Self::compute_rollup(values, &mut self.rollup, elem_dtype)
            }
            AnyValue::Vector(values) => {
                let elem_dtype = if let DataType::List(inner) = dtype {
                    *inner
                } else {
                    dtype
                };
                Self::compute_rollup(&values, &mut self.rollup, elem_dtype)
            }
            _ => match self.rollup {
                Rollup::Count => Ok(AnyValue::UInt64(1)),
                Rollup::Unique => Ok(AnyValue::Vector(vec![child_output])),
                Rollup::Quantile(ref mut q) => {
                    if let Some(v) = child_output.extract::<f32>() {
                        if v.is_finite() {
                            q.add(v);
                        }
                    } else {
                        return Ok(AnyValue::Null);
                    }

                    Ok(q.value().map(AnyValue::Float32).unwrap_or(AnyValue::Null))
                }
                _ => Ok(child_output),
            },
        }
    }
}
