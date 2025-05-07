use pyo3::{pyclass, pymethods};
use std::collections::BTreeMap;

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyEngineParam {
    pub name: String,
    pub args: BTreeMap<String, String>,
}

#[pymethods]
impl PyEngineParam {
    #[new]
    #[pyo3(signature = (name, args=None))]
    pub fn new(name: String, args: Option<BTreeMap<String, String>>) -> Self {
        Self {
            name,
            args: args.unwrap_or_default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_args(&self) -> &BTreeMap<String, String> {
        &self.args
    }

    pub fn get_arg(&self, key: &str) -> Option<&String> {
        self.args.get(key)
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyEngineBuilder {
    pub objectives: Vec<String>,
    pub survivor_selector: PyEngineParam,
    pub offspring_selector: PyEngineParam,
    pub alters: Vec<PyEngineParam>,
    pub population_size: usize,
}

#[pymethods]
impl PyEngineBuilder {
    #[new]
    #[pyo3(signature = (objectives, survivor_selector, offspring_selector, alters,
    population_size))]
    pub fn new(
        objectives: Vec<String>,
        survivor_selector: PyEngineParam,
        offspring_selector: PyEngineParam,
        alters: Vec<PyEngineParam>,
        population_size: usize,
    ) -> Self {
        Self {
            objectives,
            survivor_selector,
            offspring_selector,
            alters,
            population_size,
        }
    }

    pub fn set_population_size(&mut self, size: usize) {
        self.population_size = size;
    }

    pub fn set_survivor_selector(&mut self, selector: PyEngineParam) {
        self.survivor_selector = selector;
    }

    pub fn set_offspring_selector(&mut self, selector: PyEngineParam) {
        self.offspring_selector = selector;
    }

    pub fn set_alters(&mut self, alters: Vec<PyEngineParam>) {
        self.alters = alters;
    }
}
