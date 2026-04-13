use crate::{AnyValue, DataType, Expr, ExprProjection, ExprQuery, ExprResult, value};
use radiate_error::{radiate_bail, radiate_err};
use radiate_utils::{Slope, Statistic, WindowBuffer};
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
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    fn compute_rollup<'a>(values: &[AnyValue<'a>], rollup: Rollup) -> ExprResult<'a> {
        let mut stats = Statistic::default();
        let mut dtype = DataType::Null;

        if values.is_empty() {
            return match rollup {
                Rollup::Count => Ok(AnyValue::UInt64(0)),
                Rollup::Unique => Ok(AnyValue::Vector(vec![])),
                _ => Ok(AnyValue::Float32(0.0)),
            };
        }

        if let Rollup::First = rollup {
            return Ok(values[0].clone());
        } else if let Rollup::Last = rollup {
            return Ok(values[values.len() - 1].clone());
        } else if let Rollup::Slope = rollup {
            if values.len() < 2 {
                return Ok(AnyValue::Float32(0.0));
            }

            let slope = values
                .iter()
                .filter_map(|v| v.clone().extract::<f32>())
                .collect::<Slope<f32>>();

            return Ok(AnyValue::Float32(slope.value().unwrap_or(0.0)));
        }

        for value in values.iter() {
            if value.is_nested() {
                return Ok(AnyValue::Null);
            }

            if dtype == DataType::Null {
                dtype = value.dtype();
            } else if dtype != value.dtype() {
                radiate_bail!(Expr:
                    "Cannot compute {:?} rollup for values of different types: {:?} and {:?}",
                    rollup,
                    dtype,
                    value.dtype()
                );
            }

            if let Some(v) = value.clone().extract::<f32>() {
                stats.add(v);
            }
        }

        let result = match rollup {
            Rollup::Mean => AnyValue::Float32(stats.mean()),
            Rollup::StdDev => AnyValue::Float32(stats.std_dev().unwrap()),
            Rollup::Min => AnyValue::Float32(stats.min()),
            Rollup::Max => AnyValue::Float32(stats.max()),
            Rollup::Sum => AnyValue::Float32(stats.sum()),
            _ => AnyValue::Null,
        };

        return Ok(result.cast(&dtype).unwrap_or(AnyValue::Null));
    }
}

impl<T> ExprQuery<T> for AggExpr
where
    T: ExprProjection,
{
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprResult<'a> {
        let child_output = self.child.dispatch(input)?;

        if let Rollup::Unique = self.rollup {
            return match value::dedup(child_output) {
                Some(deduped) => Ok(deduped),
                None => Err(radiate_err!(
                    "Unique rollup is only supported for slices and vectors"
                )),
            };
        } else if let Rollup::Count = self.rollup {
            return match child_output.len() {
                Some(len) => Ok(AnyValue::UInt64(len as u64)),
                None => Ok(AnyValue::Null),
            };
        }

        match child_output {
            AnyValue::Slice(values) => Self::compute_rollup(values, self.rollup),
            AnyValue::Vector(values) => Self::compute_rollup(&values, self.rollup),
            _ => Ok(child_output),
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
    fn dispatch<'a>(&'a mut self, input: &T) -> ExprResult<'a> {
        let child_output = self.child.dispatch(input)?.into_static();

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
        Ok(AnyValue::Slice(&self.buffer.values()))
    }
}
