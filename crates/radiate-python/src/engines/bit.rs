use crate::{
    PyBitCodex, PyEngineBuilder, PyEngineParam, PyGeneration, ThreadSafePythonFn,
    conversion::ObjectValue,
};
use pyo3::{
    PyObject, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{BitChromosome, Epoch, Generation, GeneticEngine, steps::SequentialEvaluator};

#[pyclass]
pub struct PyBitEngine {
    pub engine: Option<GeneticEngine<BitChromosome, ObjectValue>>,
}

#[pymethods]
impl PyBitEngine {
    #[new]
    #[pyo3(signature = (codex, fitness_func, builder))]
    pub fn new(codex: PyBitCodex, fitness_func: PyObject, builder: PyEngineBuilder) -> Self {
        let fitness = ThreadSafePythonFn::new(fitness_func);

        let mut engine = GeneticEngine::builder()
            .codex(codex.codex)
            .num_threads(builder.num_threads)
            .evaluator(SequentialEvaluator)
            .fitness_fn(move |decoded: ObjectValue| {
                Python::with_gil(|py| fitness.call(py, decoded))
            })
            .population_size(builder.population_size);

        engine = crate::set_selector(engine, &builder.offspring_selector, true);
        engine = crate::set_selector(engine, &builder.survivor_selector, false);
        engine = crate::get_alters_with_char_gene(engine, &builder.alters);
        engine = crate::set_single_objective(engine, &builder.objectives);

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
