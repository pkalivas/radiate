use crate::{PyEngineBuilder, PyEngineParam, PyFloatCodec, PyGeneration, conversion::ObjectValue};
use pyo3::{
    PyObject, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{Epoch, FloatChromosome, Generation, GeneticEngine};

#[pyclass]
pub struct PyFloatEngine {
    pub engine: Option<GeneticEngine<FloatChromosome, ObjectValue>>,
}

#[pymethods]
impl PyFloatEngine {
    #[new]
    #[pyo3(signature = (codec, fitness_func, builder))]
    pub fn new(codec: PyFloatCodec, fitness_func: PyObject, builder: PyEngineBuilder) -> Self {
        let mut engine = crate::build_single_objective_engine(codec.codec, fitness_func, &builder);
        engine = crate::get_alters_with_float_gene(engine, &builder.alters);

        PyFloatEngine {
            engine: Some(engine.build()),
        }
    }

    pub fn run(&mut self, limits: Vec<PyEngineParam>, log: bool) -> PyResult<PyGeneration> {
        crate::run_single_objective_engine(&mut self.engine, limits, log)
    }
}

impl Into<PyGeneration> for Generation<FloatChromosome, ObjectValue> {
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
