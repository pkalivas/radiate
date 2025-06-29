use crate::{PyChromosomeType, PyGeneType};
use pyo3::{Py, PyAny, pyclass, pymethods};
use std::collections::{HashMap, HashSet};

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
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

#[pyclass]
#[derive(Debug)]
pub struct PyEngineBuilderTwo {
    pub gene_type: PyGeneType,
    pub codec: Py<PyAny>,
    pub problem_builder: Py<PyAny>,
    pub inputs: Vec<PyEngineInput>,
}

#[pymethods]
impl PyEngineBuilderTwo {
    #[new]
    pub fn new(
        gene_type: String,
        codec: Py<PyAny>,
        problem_builder: Py<PyAny>,
        inputs: Vec<PyEngineInput>,
    ) -> Self {
        let gene_type = match gene_type.as_str() {
            "float" => PyGeneType::Float,
            "int" => PyGeneType::Int,
            "bit" => PyGeneType::Bit,
            "char" => PyGeneType::Char,
            "graph" => PyGeneType::Graph,
            _ => panic!("Invalid gene type: {}", gene_type),
        };
        PyEngineBuilderTwo {
            gene_type,
            codec,
            problem_builder,
            inputs,
        }
    }

    pub fn build(&self) {
        for input in self.inputs.iter() {
            if !input.is_valid_gene(self.gene_type) {
                panic!(
                    "Input component {} of type {:?} is not valid for gene type {:?}",
                    input.component, input.input_type, self.gene_type
                );
            }

            println!("{:?}", input);
        }
    }
}
