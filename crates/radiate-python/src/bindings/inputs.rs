use crate::{AnyValue, PyGeneType, prelude::Wrap};
use pyo3::{pyclass, pymethods};
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

    pub fn get_vec_f32(&self, key: &str) -> Option<Vec<f32>> {
        self.args.get(key).and_then(|v| v.to_vec_f32())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.args.get(key).and_then(|v| match v {
            AnyValue::Bool(b) => Some(*b),
            _ => None,
        })
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
            "PyEngineInput {{ \n\tcomponent: {}, \n\tinput_type: {:?}, \n\tallowed_genes: {:?}, \n\targs: {{{}}} \n}}",
            self.component, self.input_type, self.allowed_genes, args
        )
    }
}
