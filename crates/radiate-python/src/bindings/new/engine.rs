use crate::{EngineHandle, EpochHandle, PyEngineInput, PyGeneration};
use pyo3::{PyResult, pyclass, pymethods};
use radiate::{
    Chromosome, EngineExt, Epoch, Generation, GeneticEngine, Limit, Objective, Optimize, log_ctx,
};

#[pyclass(unsendable)]
pub struct PyEngine {
    engine: Option<EngineHandle>,
}

impl PyEngine {
    pub fn new(engine: EngineHandle) -> Self {
        Self {
            engine: Some(engine),
        }
    }
}

#[pymethods]
impl PyEngine {
    pub fn run(&mut self, limits: Vec<PyEngineInput>, log: bool) -> PyResult<PyGeneration> {
        if self.engine.is_none() {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Engine has already been run",
            ));
        }

        let limits = limits
            .into_iter()
            .filter_map(|input| input.into_limit())
            .collect::<Vec<_>>();

        let engine = self.engine.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Engine has already been run")
        })?;

        let result = match engine {
            EngineHandle::Int(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Int(output)
            }
            EngineHandle::Float(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Float(output)
            }
            EngineHandle::Char(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Char(output)
            }
            EngineHandle::Bit(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::Bit(output)
            }
            EngineHandle::IntMulti(_) => panic!("Not implemented yet"),
            EngineHandle::FloatMulti(_) => panic!("Not implemented yet"),
            EngineHandle::GraphRegression(eng) => {
                let output = run_single_objective_engine(eng, limits, log);
                EpochHandle::GraphRegression(output)
            }
        };

        Ok(match result {
            EpochHandle::Int(epoch) => epoch.into(),
            EpochHandle::Float(epoch) => epoch.into(),
            EpochHandle::Char(epoch) => epoch.into(),
            EpochHandle::Bit(epoch) => epoch.into(),
            EpochHandle::IntMulti(_) => panic!("Not implemented yet"),
            EpochHandle::FloatMulti(_) => panic!("Not implemented yet"),
            EpochHandle::GraphRegression(epoch) => epoch.into(),
        })
    }
}

fn run_single_objective_engine<C, T>(
    mut engine: GeneticEngine<C, T, Generation<C, T>>,
    limits: Vec<Limit>,
    log: bool,
) -> Generation<C, T>
where
    C: Chromosome + Clone,
    T: Clone + Send + Sync + 'static,
{
    engine.run(|epoch| {
        if log {
            log_ctx!(epoch);
        }

        if epoch.index() == 10 {
            return true;
        }

        false
    })
    // engine
    //     .iter()
    //     .inspect(|epoch| {
    //         if log {
    //             log_ctx!(epoch);
    //         }
    //     })
    //     .skip_while(|epoch| {
    //         limits.iter().all(|limit| match limit {
    //             Limit::Generation(lim) => epoch.index() < *lim,
    //             Limit::Score(lim) => match epoch.objective() {
    //                 Objective::Single(opt) => match opt {
    //                     Optimize::Minimize => epoch.score().as_f32() > *lim,
    //                     Optimize::Maximize => epoch.score().as_f32() < *lim,
    //                 },
    //                 Objective::Multi(_) => false,
    //             },
    //             Limit::Seconds(val) => return epoch.seconds() < *val,
    //         })
    //     })
    //     .take(1)
    //     .last()
    //     .expect("No generation found that meets the limits")
}

// fn run_multi_objective_engine<C, T>(
//     engine: &mut Option<GeneticEngine<C, T, ParetoGeneration<C>>>,
//     limits: Vec<PyLimit>,
//     _: bool,
// ) -> PyResult<PyGeneration>
// where
//     C: Chromosome + Clone,
//     T: Clone + Send + Sync + 'static,
//     ParetoGeneration<C>: Into<PyGeneration>,
// {
//     engine
//         .take()
//         .map(|engine| {
//             engine
//                 .iter()
//                 .skip_while(|epoch| {
//                     limits.iter().all(|limit| match limit {
//                         PyLimit::Generation(lim) => epoch.index() < *lim,
//                         PyLimit::Score(_) => false,
//                         PyLimit::Seconds(val) => return epoch.seconds() < *val,
//                     })
//                 })
//                 .take(1)
//                 .last()
//                 .map(|epoch| epoch.into())
//         })
//         .flatten()
//         .ok_or(PyRuntimeError::new_err(
//             "No generation found that meets the limits",
//         ))
// }
