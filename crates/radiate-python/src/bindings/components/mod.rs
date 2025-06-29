mod alterer;
mod diversity;
mod selector;

use std::collections::HashMap;

pub use alterer::PyAlterer;
pub use diversity::PyDiversity;
use pyo3::{pyclass, pymethods};
pub use selector::PySelector;

use crate::{PyChromosomeType, PyGeneType};

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub enum InputType {
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
pub struct PyEngineParam {
    pub name: String,
    pub input_type: InputType,
    pub gene_type: PyGeneType,
    pub args: HashMap<String, String>,
    pub allowed_genes: Vec<PyGeneType>,
    pub allowed_chromosomes: Vec<PyChromosomeType>,
}

#[pymethods]
impl PyEngineParam {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn input_type(&self) -> InputType {
        self.input_type.clone()
    }

    pub fn gene_type(&self) -> PyGeneType {
        self.gene_type.clone()
    }

    pub fn args(&self) -> HashMap<String, String> {
        self.args.clone()
    }

    pub fn allowed_genes(&self) -> Vec<PyGeneType> {
        self.allowed_genes.clone()
    }

    pub fn allowed_chromosomes(&self) -> Vec<PyChromosomeType> {
        self.allowed_chromosomes.clone()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "EngineParam(name={}, input_type={:?}, gene_type={:?}, args={:?}, allowed_genes={:?}, allowed_chromosomes={:?})",
            self.name,
            self.input_type,
            self.gene_type,
            self.args,
            self.allowed_genes,
            self.allowed_chromosomes
        )
    }
    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}
