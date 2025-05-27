use crate::{
    PyEngineBuilder, PyEngineParam, PyGeneration, codec::PyCharCodec, conversion::ObjectValue,
};
use pyo3::{
    PyObject, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{CharChromosome, Epoch, Generation, GeneticEngine};

#[pyclass]
pub struct PyCharEngine {
    pub engine: Option<GeneticEngine<CharChromosome, ObjectValue>>,
}

#[pymethods]
impl PyCharEngine {
    #[new]
    #[pyo3(signature = (codec, fitness_func, builder))]
    pub fn new(codec: PyCharCodec, fitness_func: PyObject, builder: PyEngineBuilder) -> Self {
        let mut engine = crate::build_single_objective_engine(codec.codec, fitness_func, &builder);
        engine = crate::get_alters_with_char_gene(engine, &builder.alters);

        PyCharEngine {
            engine: Some(engine.build()),
        }
    }

    pub fn run(&mut self, limits: Vec<PyEngineParam>, log: bool) -> PyResult<PyGeneration> {
        crate::run_single_objective_engine(&mut self.engine, limits, log)
    }
}

impl Into<PyGeneration> for Generation<CharChromosome, ObjectValue> {
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
