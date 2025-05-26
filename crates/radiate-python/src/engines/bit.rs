use crate::{PyBitCodex, PyEngineBuilder, PyEngineParam, PyGeneration, conversion::ObjectValue};
use pyo3::{
    PyObject, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{BitChromosome, Epoch, Generation, GeneticEngine};

#[pyclass]
pub struct PyBitEngine {
    pub engine: Option<GeneticEngine<BitChromosome, ObjectValue>>,
}

#[pymethods]
impl PyBitEngine {
    #[new]
    #[pyo3(signature = (codex, fitness_func, builder))]
    pub fn new(codex: PyBitCodex, fitness_func: PyObject, builder: PyEngineBuilder) -> Self {
        let mut engine = crate::build_single_objective_engine(codex.codex, fitness_func, &builder);
        engine = crate::get_alters_with_char_gene(engine, &builder.alters);

        PyBitEngine {
            engine: Some(engine.build()),
        }
    }

    pub fn run(&mut self, limits: Vec<PyEngineParam>, log: bool) -> PyResult<PyGeneration> {
        crate::run_single_objective_engine(&mut self.engine, limits, log)
    }
}

impl Into<PyGeneration> for Generation<BitChromosome, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
            }
        })
    }
}
