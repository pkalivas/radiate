use crate::{
    PyEngineInput, PyEngineInputType, PyExpr,
    bindings::converters::{FromPyEngineInput, Registry},
};
use radiate::{Limit, RadiateResult};
use radiate_error::radiate_bail;
use std::{collections::VecDeque, time::Duration};

fn limits() -> Registry<Limit> {
    use crate::constants::components;
    Registry {
        kind: "limit",
        entries: &[
            (components::SCORE_LIMIT, score),
            (components::GENERATIONS_LIMIT, generations),
            (components::SECONDS_LIMIT, seconds),
            (components::CONVERGENCE_LIMIT, convergence),
            (components::EXPR_LIMIT, expr),
        ],
    }
}

impl FromPyEngineInput for Limit {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        input.expect_type(PyEngineInputType::Limit)?;
        limits().convert(input)
    }
}

fn score(input: &PyEngineInput) -> RadiateResult<Limit> {
    // A single objective passes a number, multi-objective passes one per objective.
    if let Ok(score) = input.extract::<f64>("score") {
        Ok(Limit::Score(score.into()))
    } else if let Ok(scores) = input.extract::<Vec<f64>>("score") {
        Ok(Limit::Score(scores.into()))
    } else {
        radiate_bail!(Builder: "Score limit expects 'score' to be a number or a list of numbers")
    }
}

fn generations(input: &PyEngineInput) -> RadiateResult<Limit> {
    let generations = input.extract::<i64>("generations")?;
    Ok(Limit::Generation(generations as usize))
}

fn seconds(input: &PyEngineInput) -> RadiateResult<Limit> {
    let seconds = input.extract::<f64>("seconds")?;
    Ok(Limit::Seconds(Duration::from_secs_f64(seconds)))
}

fn convergence(input: &PyEngineInput) -> RadiateResult<Limit> {
    let window = input.extract::<i64>("window")? as usize;
    let epsilon = input.extract::<f64>("epsilon")?;
    Ok(Limit::Convergence(
        window,
        epsilon as f32,
        VecDeque::with_capacity(window),
    ))
}

fn expr(input: &PyEngineInput) -> RadiateResult<Limit> {
    let expr = input.extract::<PyExpr>("expr")?;
    Ok(Limit::Expr(expr.into()))
}
