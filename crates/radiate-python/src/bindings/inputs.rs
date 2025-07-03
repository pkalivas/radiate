use crate::{AnyValue, PyGeneType, prelude::Wrap};
use pyo3::{pyclass, pymethods};
use radiate::{Executor, Limit};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum PyEngineInputType {
    Alterer,
    OffspringSelector,
    SurvivorSelector,
    Diversity,
    Objective,
    Limit,
    PopulationSize,
    OffspringFraction,
    MaxSpeciesAge,
    MaxPhenotypeAge,
    FrontRange,
    Executor,
    Evaluator,
    SpeciesThreshold,
}

#[pyclass]
#[derive(Clone)]
pub struct PyEngineInput {
    pub component: String,
    pub input_type: PyEngineInputType,
    pub allowed_genes: HashSet<PyGeneType>,
    pub args: HashMap<String, AnyValue<'static>>,
}

#[pymethods]
impl PyEngineInput {
    #[new]
    pub fn new(
        component: String,
        input_type: PyEngineInputType,
        allowed_genes: HashSet<PyGeneType>,
        args: HashMap<String, Wrap<AnyValue<'_>>>,
    ) -> Self {
        PyEngineInput {
            component,
            input_type,
            allowed_genes,
            args: args
                .into_iter()
                .map(|(k, v)| (k, v.0.into_static()))
                .collect(),
        }
    }

    pub fn component(&self) -> String {
        self.component.clone()
    }

    pub fn input_type(&self) -> PyEngineInputType {
        self.input_type.clone()
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl PyEngineInput {
    pub fn get(&self, key: &str) -> Option<&AnyValue<'static>> {
        self.args.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.args.get(key).and_then(|v| v.to_string())
    }

    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.args.get(key).and_then(|v| v.to_i32())
    }

    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.args.get(key).and_then(|v| v.to_f32())
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.args.get(key).and_then(|v| v.to_f64())
    }

    pub fn get_usize(&self, key: &str) -> Option<usize> {
        self.args.get(key).and_then(|v| v.to_usize())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.args.get(key).and_then(|v| match v {
            AnyValue::Bool(b) => Some(*b),
            _ => None,
        })
    }
}

impl Into<Option<Executor>> for PyEngineInput {
    fn into(self) -> Option<Executor> {
        if self.input_type != PyEngineInputType::Executor {
            return None;
        }

        Some(match self.component.as_str() {
            "Serial" => Executor::Serial,
            "FixedSizedWorkerPool" => {
                let num_workers = self.get_usize("num_workers").unwrap_or(1);

                Executor::FixedSizedWorkerPool(num_workers)
            }
            "WorkerPool" => Executor::WorkerPool,
            _ => panic!("Executor type {} not yet implemented", self.component),
        })
    }
}

impl Into<Option<Limit>> for PyEngineInput {
    fn into(self) -> Option<Limit> {
        if self.input_type != PyEngineInputType::Limit {
            return None;
        }

        if let Some(generation) = self.get_usize("generations") {
            return Some(Limit::Generation(generation));
        }

        if let Some(sec) = self.get_f64("seconds") {
            return Some(Limit::Seconds(sec));
        }

        if let Some(score) = self.get_f32("score") {
            return Some(Limit::Score(score));
        }

        None
    }
}

impl Debug for PyEngineInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut args = self
            .args
            .iter()
            .map(|(k, v)| format!("\t\t{}: {:?}", k, v))
            .collect::<Vec<String>>()
            .join("\n");
        if args.len() > 0 {
            args = format!("\n{}", args);
            args.push('\n');
        }
        write!(
            f,
            "PyEngineInput {{ \n\tcomponent: {}, \n\tinput_type: {:?}, \n\tallowed_genes: {:?}, \n\ttemp: {{{}}} \n}}",
            self.component, self.input_type, self.allowed_genes, args
        )
    }
}
