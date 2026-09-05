use crate::{
    Expr, ExprResult, ProjectExpr,
    ops::{BinaryOp, RollupOp, ScheduleOp, TrinaryOp, UnaryOp},
};
use radiate_error::radiate_bail;
use radiate_utils::{AnyValue, DataType, Quantile, Slope, Statistic, WindowBuffer, dedup_slice};

pub(crate) fn try_schedule(op: &mut ScheduleOp) -> ExprResult<'static> {
    match op {
        ScheduleOp::Interval { count, limit } => {
            *count += 1;
            if *count >= *limit {
                *count = 0;
                Ok(AnyValue::Bool(true))
            } else {
                Ok(AnyValue::Bool(false))
            }
        }
        ScheduleOp::Duration { last, interval } => {
            let now = std::time::Instant::now();
            if let Some(last) = last {
                if now.duration_since(*last) >= *interval {
                    *last = now;
                    Ok(AnyValue::Bool(true))
                } else {
                    Ok(AnyValue::Bool(false))
                }
            } else {
                *last = Some(now);
                Ok(AnyValue::Bool(true))
            }
        }
    }
}

pub(crate) fn unary_eval<'a>(
    expr: &'a mut Expr,
    op: &mut UnaryOp,
    input: &'a impl ProjectExpr<'a>,
) -> ExprResult<'a> {
    let value = expr.evaluate(input)?;
    match op {
        UnaryOp::Not => match value {
            AnyValue::Bool(b) => Ok(AnyValue::Bool(!b)),
            _ => radiate_bail!(Expr: "Logical NOT is only supported for boolean types"),
        },
        UnaryOp::Neg => match value.extract::<f32>() {
            Some(v) => Ok(AnyValue::Float32(-v)),
            None => radiate_bail!(Expr: "Negation is only supported for numeric types"),
        },
        UnaryOp::Abs => match value.extract::<f32>() {
            Some(v) => Ok(AnyValue::Float32(v.abs())),
            None => radiate_bail!(Expr: "Absolute value is only supported for numeric types"),
        },
        UnaryOp::Cast(to) => match value.clone().cast(to) {
            Some(v) => Ok(v),
            None => radiate_bail!(Expr: "Failed to cast value {:?} to type {:?}", value, to),
        },
        UnaryOp::Debug => {
            println!("{:?}", value);
            Ok(value)
        }
        UnaryOp::Affine { scale, bias } => match value.extract::<f32>() {
            Some(x) if x.is_finite() => Ok(AnyValue::Float32(*scale * x + *bias)),
            _ => Ok(AnyValue::Null),
        },
        UnaryOp::Stagnation {
            epsilon,
            last_value,
            count,
        } => {
            let current = match value.extract::<f32>() {
                Some(v) if v.is_finite() => v,
                _ => return Ok(AnyValue::Null),
            };

            match last_value {
                None => {
                    *last_value = Some(current);
                    *count = 0;
                }
                Some(last) => {
                    if (current - *last).abs() > *epsilon {
                        *last_value = Some(current);
                        *count = 0;
                    } else {
                        *count = count.saturating_add(1);
                    }
                }
            }

            Ok(AnyValue::UInt32(*count))
        }
    }
}

pub(crate) fn binary_eval<'a>(
    lhs: &'a mut Expr,
    rhs: &'a mut Expr,
    op: &BinaryOp,
    input: &'a impl ProjectExpr<'a>,
) -> ExprResult<'a> {
    if let BinaryOp::Coalesce = op {
        let lhs_value = lhs.evaluate(input)?;
        let is_bad = match lhs_value.extract::<f32>() {
            Some(v) => !v.is_finite(),
            None => matches!(lhs_value, AnyValue::Null),
        };
        return if is_bad {
            rhs.evaluate(input)
        } else {
            Ok(lhs_value)
        };
    }

    let lhs = lhs.evaluate(input)?;
    let rhs = rhs.evaluate(input)?;

    let result = match op {
        BinaryOp::Coalesce => unreachable!("handled above"),
        BinaryOp::Add => lhs + rhs,
        BinaryOp::Sub => lhs - rhs,
        BinaryOp::Mul => lhs * rhs,
        BinaryOp::Div => lhs / rhs,
        BinaryOp::Lt => AnyValue::Bool(lhs < rhs),
        BinaryOp::Lte => AnyValue::Bool(lhs <= rhs),
        BinaryOp::Gt => AnyValue::Bool(lhs > rhs),
        BinaryOp::Gte => AnyValue::Bool(lhs >= rhs),
        BinaryOp::Eq => AnyValue::Bool(lhs == rhs),
        BinaryOp::Ne => AnyValue::Bool(lhs != rhs),
        BinaryOp::And => lhs & rhs,
        BinaryOp::Or => lhs | rhs,
        BinaryOp::Mod => lhs % rhs,
        BinaryOp::Pow => radiate_utils::pow_anyvalue(&lhs, &rhs)?,
        BinaryOp::Min => match (lhs.extract::<f32>(), rhs.extract::<f32>()) {
            (Some(a), Some(b)) => AnyValue::Float32(a.min(b)),
            _ => radiate_bail!(Expr: "Min requires numeric operands"),
        },
        BinaryOp::Max => match (lhs.extract::<f32>(), rhs.extract::<f32>()) {
            (Some(a), Some(b)) => AnyValue::Float32(a.max(b)),
            _ => radiate_bail!(Expr: "Max requires numeric operands"),
        },
    };

    Ok(result)
}

pub(crate) fn trinary_eval<'a>(
    first: &'a mut Expr,
    second: &'a mut Expr,
    third: &'a mut Expr,
    op: &TrinaryOp,
    input: &'a impl ProjectExpr<'a>,
) -> ExprResult<'a> {
    match op {
        TrinaryOp::If => {
            let cond = match first.evaluate(input)? {
                AnyValue::Bool(b) => b,
                _ => radiate_bail!(Expr: "Condition must be a boolean"),
            };
            if cond {
                second.evaluate(input)
            } else {
                third.evaluate(input)
            }
        }
        TrinaryOp::Clamp => {
            let value = first.evaluate(input)?.extract::<f32>();
            let min = second.evaluate(input)?.extract::<f32>();
            let max = third.evaluate(input)?.extract::<f32>();

            let (min_v, max_v) = match (min, max) {
                (Some(a), Some(b)) => (a, b),
                _ => radiate_bail!(Expr: "Clamp bounds must be numeric"),
            };

            let result = match value {
                Some(v) if v.is_finite() => v.clamp(min_v, max_v),
                _ => min_v,
            };
            Ok(AnyValue::Float32(result))
        }
    }
}

pub(crate) fn rolling_eval<'a>(
    child: &'a mut Expr,
    input: &'a impl ProjectExpr<'a>,
    buffer: &'a mut WindowBuffer<AnyValue<'static>>,
) -> ExprResult<'a> {
    let result = child.evaluate(input)?;
    if result.is_nested() {
        radiate_bail!(Expr: "Nested result not allowed in rolling window");
    }

    buffer.push(result.into_static());
    Ok(AnyValue::Slice(buffer.as_slice()))
}

pub(crate) fn reduce_eval<'a>(
    child: &'a mut Expr,
    input: &'a impl ProjectExpr<'a>,
    op: &RollupOp,
) -> ExprResult<'a> {
    let child_output = child.evaluate(input)?;
    let dtype = child_output.dtype();

    match child_output {
        AnyValue::Slice(values) => {
            let elem_dtype = if let DataType::List(inner) = dtype {
                *inner
            } else {
                dtype
            };
            rollup(op, &values, elem_dtype)
        }
        AnyValue::Vector(values) => {
            let elem_dtype = if let DataType::List(inner) = dtype {
                *inner
            } else {
                dtype
            };
            rollup(op, &values, elem_dtype)
        }
        _ => {
            radiate_bail!(Expr: "ReduceExpr expected a Slice or Vector, got: {:?}", child_output)
        }
    }
}

fn rollup<'a>(op: &RollupOp, values: &[AnyValue<'a>], dtype: DataType) -> ExprResult<'a> {
    if values.is_empty() {
        return match op {
            RollupOp::Count => Ok(AnyValue::UInt64(0)),
            _ => Ok(AnyValue::Float32(0.0)),
        };
    }

    if values.len() == 1 {
        return match op {
            RollupOp::Count => Ok(AnyValue::UInt64(1)),
            RollupOp::Unique => Ok(AnyValue::Vector(values.to_vec())),
            _ => Ok(values[0].clone()),
        };
    }

    if let RollupOp::Unique = op {
        return Ok(dedup_slice(values));
    } else if let RollupOp::Count = op {
        return Ok(AnyValue::UInt64(values.len() as u64));
    } else if let RollupOp::First = op {
        return Ok(values[0].clone());
    } else if let RollupOp::Last = op {
        return Ok(values[values.len() - 1].clone());
    } else if let RollupOp::Slope = op {
        if values.len() < 2 {
            return Ok(AnyValue::Float32(0.0));
        }

        let slope = values
            .iter()
            .filter_map(|v| v.extract::<f32>())
            .collect::<Slope<f32>>();

        return Ok(AnyValue::Float32(slope.value().unwrap_or(0.0)));
    } else if let RollupOp::Quantile(quantile) = op {
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

    let result = match op {
        RollupOp::Mean => AnyValue::Float32(stats.mean()),
        RollupOp::StdDev => AnyValue::Float32(stats.std_dev().unwrap()),
        RollupOp::Min => AnyValue::Float32(stats.min()),
        RollupOp::Max => AnyValue::Float32(stats.max()),
        RollupOp::Sum => AnyValue::Float32(stats.sum()),
        RollupOp::Count => AnyValue::UInt64(stats.count() as u64),
        _ => AnyValue::Null,
    };

    Ok(result.cast(&dtype).unwrap_or(AnyValue::Null))
}
