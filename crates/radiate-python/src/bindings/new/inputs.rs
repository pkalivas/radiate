use crate::PyGeneType;
use pyo3::{pyclass, pymethods};
use radiate::Limit;
use std::collections::{HashMap, HashSet};

#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum PyEngineInputType {
    Alterer,
    OffspringSelector,
    SurvivorSelector,
    Diversity,
    Objective,
    Limit,
    Subscriber,
    PopulationSize,
    OffspringFraction,
    MaxSpeciesAge,
    MaxPhenotypeAge,
    FrontRange,
    Codec,
    Executor,
    Problem,
    SpeciesThreshold,
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyEngineInput {
    pub component: String,
    pub input_type: PyEngineInputType,
    pub args: HashMap<String, String>,
    pub allowed_genes: HashSet<PyGeneType>,
}

#[pymethods]
impl PyEngineInput {
    #[new]
    pub fn new(
        component: String,
        input_type: PyEngineInputType,
        args: HashMap<String, String>,
        allowed_genes: HashSet<PyGeneType>,
    ) -> Self {
        PyEngineInput {
            component,
            input_type,
            args,
            allowed_genes,
        }
    }

    pub fn component(&self) -> String {
        self.component.clone()
    }

    pub fn input_type(&self) -> PyEngineInputType {
        self.input_type.clone()
    }

    pub fn args(&self) -> HashMap<String, String> {
        self.args.clone()
    }

    pub fn allowed_genes(&self) -> HashSet<PyGeneType> {
        self.allowed_genes.clone()
    }

    pub fn is_valid_gene(&self, gene_type: PyGeneType) -> bool {
        self.allowed_genes.contains(&gene_type)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "EngineParam(component={}, input_type={:?}, args={:?}, allowed_genes={:?})",
            self.component, self.input_type, self.args, self.allowed_genes
        )
    }
    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl PyEngineInput {
    pub fn into_limit(self) -> Option<Limit> {
        if self.input_type != PyEngineInputType::Limit {
            return None;
        }

        if let Some(generation) = self.args.get("generation") {
            if let Ok(g) = generation.parse::<usize>() {
                return Some(Limit::Generation(g));
            }
        }

        if let Some(sec) = self.args.get("seconds") {
            if let Ok(s) = sec.parse::<f64>() {
                return Some(Limit::Seconds(s));
            }
        }

        if let Some(score) = self.args.get("score") {
            if let Ok(s) = score.parse::<f32>() {
                return Some(Limit::Score(s));
            }
        }

        None
    }
}
