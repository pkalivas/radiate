use crate::{
    AnyValue, PyEngineParam, PySelector, ThreadSafePythonFn, get_alters_with_arithmetic_gene,
};
use pyo3::{
    IntoPyObjectExt, Py, PyAny, PyObject, PyResult, Python, pyclass, pymethods,
    types::PyDictMethods,
};
use radiate::{
    Engine, EngineExt, Epoch, FloatChromosome, FloatCodex, GeneticEngine, log_ctx,
    steps::SequentialEvaluator,
};

pub trait PyEngineTrait {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn into_inner(self: Box<Self>) -> Box<dyn std::any::Any>;
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[pyclass(unsendable)]
pub struct PyEngine {
    pub engine: Option<Box<dyn PyEngineTrait>>,
}

#[pymethods]
impl PyEngine {
    pub fn run(&mut self, generations: usize) {
        let maybe_engine = self.engine.take();
        if let Some(mut engine) = maybe_engine {
            let engine = engine
                .as_any_mut()
                .downcast_mut::<GeneticEngine<FloatChromosome, Vec<Vec<f32>>>>()
                .unwrap();

            engine.run(|epoch| {
                log_ctx!(epoch);
                epoch.index() > generations
            });
        }
    }

    pub fn next<'py>(&mut self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        if let Some(engine) = self.engine.as_mut() {
            let engine = engine
                .as_any_mut()
                .downcast_mut::<GeneticEngine<FloatChromosome, Vec<Vec<f32>>>>()
                .unwrap();

            let epoch = engine.next();

            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("index", epoch.index()).unwrap();
            dict.set_item("best", epoch.value()).unwrap();

            return dict.into_py_any(py);
        } else {
            Err(pyo3::exceptions::PyStopIteration::new_err("No more epochs"))
        }
    }
}

#[pymethods]
impl PyEngine {
    #[staticmethod]
    #[pyo3(signature = (num_genes, num_chromosomes, objective, fitness_fn, range=None, bounds=None,
    survivor_selector=None, offspring_selector=None, alters=None, size=None))]
    pub fn try_build_float_engine<'py>(
        num_genes: usize,
        num_chromosomes: usize,
        objective: Option<Vec<String>>,
        fitness_fn: PyObject,
        range: Option<(f32, f32)>,
        bounds: Option<(f32, f32)>,
        survivor_selector: Option<PySelector>,
        offspring_selector: Option<PySelector>,
        alters: Option<Vec<PyEngineParam>>,
        size: Option<usize>,
    ) -> PyResult<PyEngine> {
        let fitness_fn = ThreadSafePythonFn::new(fitness_fn);

        let range = range.map(|(min, max)| min..max).unwrap_or(0.0..1.0);
        let bounds = bounds.map(|(min, max)| min..max).unwrap_or(range.clone());

        let codex = FloatCodex::matrix(num_chromosomes, num_genes, range).with_bounds(bounds);

        let mut builder = GeneticEngine::builder()
            .codex(codex)
            .minimizing()
            .evaluator(SequentialEvaluator)
            .fitness_fn(move |decoded: Vec<Vec<f32>>| {
                Python::with_gil(|py| {
                    let wrapped_decoded = AnyValue::from(decoded);
                    let result = fitness_fn.call(py, wrapped_decoded);

                    result
                })
            });

        if let Some(size) = size {
            builder = builder.population_size(size);
        }

        if let Some(surv_selector) = survivor_selector {
            builder = crate::set_selector(builder, surv_selector, false);
        }

        if let Some(offs_selector) = offspring_selector {
            builder = crate::set_selector(builder, offs_selector, true);
        }

        if let Some(alters) = alters {
            let alters = get_alters_with_arithmetic_gene(alters);
            builder = builder.alter(alters);
        }

        if let Some(objectives) = objective {
            builder = builder.minimizing();
        }

        Ok(PyEngine {
            engine: Some(Box::new(builder.build())),
        })
    }
}

macro_rules! impl_py_eng {
    ($engine:ty) => {
        impl PyEngineTrait for $engine {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn into_inner(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }
        }
    };
}

impl_py_eng!(GeneticEngine<FloatChromosome, Vec<Vec<f32>>>);
