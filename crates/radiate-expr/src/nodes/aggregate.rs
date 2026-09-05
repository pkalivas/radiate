use crate::{EvalExpr, Expr, ExprResult, ExprSelect};
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
    Quantile(f32),
}

impl Rollup {
    pub fn reduce<'a>(&self, values: &[AnyValue<'a>], dtype: DataType) -> ExprResult<'a> {
        if values.is_empty() {
            return match self {
                Rollup::Count => Ok(AnyValue::UInt64(0)),
                _ => Ok(AnyValue::Float32(0.0)),
            };
        }

        if values.len() == 1 {
            return match self {
                Rollup::Count => Ok(AnyValue::UInt64(1)),
                Rollup::Unique => Ok(AnyValue::Vector(values.to_vec())),
                _ => Ok(values[0].clone()),
            };
        }

        if let Rollup::Unique = self {
            return Ok(dedup_slice(values));
        } else if let Rollup::Count = self {
            return Ok(AnyValue::UInt64(values.len() as u64));
        } else if let Rollup::First = self {
            return Ok(values[0].clone());
        } else if let Rollup::Last = self {
            return Ok(values[values.len() - 1].clone());
        } else if let Rollup::Slope = self {
            if values.len() < 2 {
                return Ok(AnyValue::Float32(0.0));
            }

            let slope = values
                .iter()
                .filter_map(|v| v.extract::<f32>())
                .collect::<Slope<f32>>();

            return Ok(AnyValue::Float32(slope.value().unwrap_or(0.0)));
        } else if let Rollup::Quantile(quantile) = self {
            if values.len() < 2 {
                return Ok(AnyValue::Float32(0.0));
            }

            let mut quantile = Quantile::<f32>::new(*quantile);
            for v in values.iter().filter_map(|v| v.extract::<f32>()) {
                quantile.add(v);
            }
            let result = quantile.value().unwrap_or(0.0);
            return Ok(AnyValue::Float32(result));
        }

        let stats = values
            .iter()
            .filter_map(|val| val.extract::<f32>())
            .collect::<Statistic>();

        let result = match self {
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub struct ReduceExpr {
    pub(crate) child: Box<Expr>,
    pub(crate) rollup: Rollup,
}

impl ReduceExpr {
    pub fn new(child: Expr, rollup: Rollup) -> Self {
        Self {
            child: Box::new(child),
            rollup,
        }
    }
}

impl<'a, I: ExprSelect<'a>> EvalExpr<'a, I> for ReduceExpr {
    fn evaluate(&'a mut self, input: &'a I) -> ExprResult<'a, AnyValue<'a>> {
        let child_output = self.child.evaluate(input)?;
        let dtype = child_output.dtype();

        match child_output {
            AnyValue::Slice(values) => {
                let elem_dtype = if let DataType::List(inner) = dtype {
                    *inner
                } else {
                    dtype
                };
                self.rollup.reduce(&values, elem_dtype)
            }
            AnyValue::Vector(values) => {
                let elem_dtype = if let DataType::List(inner) = dtype {
                    *inner
                } else {
                    dtype
                };
                self.rollup.reduce(&values, elem_dtype)
            }
            _ => {
                radiate_bail!(Expr: "ReduceExpr expected a Slice or Vector, got: {:?}", child_output)
            }
        }
    }
}

impl Debug for ReduceExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.rollup)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub struct RollingExpr {
    pub(crate) child: Box<Expr>,
    pub(crate) buffer: WindowBuffer<AnyValue<'static>>,
}

impl RollingExpr {
    pub fn new(child: Expr, window_size: usize) -> Self {
        Self {
            child: Box::new(child),
            buffer: WindowBuffer::with_capacity(window_size),
        }
    }
}

impl<'a, I: ExprSelect<'a>> EvalExpr<'a, I> for RollingExpr {
    fn evaluate(&'a mut self, input: &'a I) -> ExprResult<'a, AnyValue<'a>> {
        let child_output = self.child.evaluate(input)?;
        self.buffer.push(child_output.into_static());
        Ok(AnyValue::Slice(self.buffer.as_slice()))
    }
}

impl Debug for RollingExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "window_size: {}", self.buffer.len())
    }
}
