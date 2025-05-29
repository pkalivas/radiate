mod bit;
mod char;
mod float;
mod int;

pub use bit::PyBitEngine;
pub use char::PyCharEngine;
pub use float::PyFloatEngine;
pub use int::PyIntEngine;

use crate::{Limit, PyEngineParam, PyGeneration};
use pyo3::PyResult;
use radiate::{Chromosome, Epoch, Generation, GeneticEngine, Objective, Optimize, log_ctx};
use std::fmt::Debug;

pub(crate) fn run_single_objective_engine<C, T>(
    engine: &mut Option<GeneticEngine<C, T, Generation<C, T>>>,
    limits: Vec<PyEngineParam>,
    log: bool,
) -> PyResult<PyGeneration>
where
    C: Chromosome + Clone,
    T: Debug + Clone + Send + Sync,
    Generation<C, T>: Into<PyGeneration>,
{
    let lims = limits.into_iter().map(Limit::from).collect::<Vec<_>>();

    engine
        .take()
        .map(|engine| {
            engine
                .iter()
                .inspect(|epoch| {
                    if log {
                        log_ctx!(epoch);
                    }
                })
                .skip_while(|epoch| {
                    lims.iter().all(|limit| match limit {
                        Limit::Generations(lim) => epoch.index() < *lim,
                        Limit::Score(lim) => match epoch.objective() {
                            Objective::Single(opt) => match opt {
                                Optimize::Minimize => epoch.score().as_f32() > *lim,
                                Optimize::Maximize => epoch.score().as_f32() < *lim,
                            },
                            Objective::Multi(_) => false,
                        },
                        Limit::Seconds(val) => return epoch.seconds() < *val,
                    })
                })
                .take(1)
                .last()
                .inspect(|epoch| {
                    if log {
                        println!("{:?}", epoch);
                    }
                })
                .map(|epoch| epoch.into())
        })
        .flatten()
        .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
            "No generation found that meets the limits",
        ))
}
