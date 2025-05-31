use pyo3::{PyObject, pyclass, pymethods};
use std::{collections::BTreeMap, ops::Range};

use crate::ObjectValue;

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
pub struct PyFunc {
    pub func: ObjectValue,
}

#[pymethods]
impl PyFunc {
    #[new]
    pub fn new(func: ObjectValue) -> Self {
        Self { func }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyEngineBuilder {
    pub objectives: Vec<String>,
    pub survivor_selector: PyEngineParam,
    pub offspring_selector: PyEngineParam,
    pub alters: Vec<PyEngineParam>,
    pub diversity: Option<PyEngineParam>,
    pub population_size: usize,
    pub offspring_fraction: f32,
    pub species_threshold: f32,
    pub num_threads: usize,
    pub max_phenotype_age: usize,
    pub max_species_age: usize,
    pub event_handlers: Option<Vec<PyFunc>>,
    pub front_range: Range<usize>,
}

#[pymethods]
impl PyEngineBuilder {
    #[new]
    #[pyo3(signature = (objectives, survivor_selector, offspring_selector, alters,
    population_size, offspring_fraction, num_threads, front_range, species_threshold, max_phenotype_age, max_species_age, event_handlers=None, diversity=None))]
    pub fn new(
        objectives: Vec<String>,
        survivor_selector: PyEngineParam,
        offspring_selector: PyEngineParam,
        alters: Vec<PyEngineParam>,
        population_size: usize,
        offspring_fraction: f32,
        num_threads: usize,
        front_range: (usize, usize),
        species_threshold: f32,
        max_phenotype_age: usize,
        max_species_age: usize,
        event_handlers: Option<Vec<PyFunc>>,
        diversity: Option<PyEngineParam>,
    ) -> Self {
        Self {
            objectives,
            survivor_selector,
            offspring_selector,
            alters,
            population_size,
            offspring_fraction,
            num_threads,
            front_range: front_range.0..front_range.1,
            species_threshold,
            max_phenotype_age,
            max_species_age,
            event_handlers,
            diversity,
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

    pub fn set_offspring_fraction(&mut self, fraction: f32) {
        self.offspring_fraction = fraction;
    }

    pub fn set_objectives(&mut self, objectives: Vec<String>) {
        self.objectives = objectives;
    }

    pub fn set_num_threads(&mut self, num_threads: usize) {
        self.num_threads = num_threads;
    }

    pub fn set_front_range(&mut self, front_range: (usize, usize)) {
        self.front_range = front_range.0..front_range.1;
    }

    pub fn set_diversity(&mut self, diversity: Option<PyEngineParam>) {
        self.diversity = diversity;
    }

    pub fn set_species_threshold(&mut self, threshold: f32) {
        self.species_threshold = threshold;
    }

    pub fn set_max_phenotype_age(&mut self, age: usize) {
        self.max_phenotype_age = age;
    }

    pub fn set_max_species_age(&mut self, age: usize) {
        self.max_species_age = age;
    }

    pub fn set_event_handlers(&mut self, handlers: Option<Vec<PyFunc>>) {
        self.event_handlers = handlers;
    }
}
