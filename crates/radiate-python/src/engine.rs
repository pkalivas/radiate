use crate::{
    AnyValue, DataType, Field,
    conversion::{Wrap, py_object_to_any_value},
};
use pyo3::{PyObject, PyResult, Python, pyclass, pymethods};
use radiate::{
    Chromosome, Ecosystem, EngineExt, Epoch, FloatChromosome, FloatCodex, Gene, GeneticEngine,
    Genotype, Problem, Score,
    steps::{Evaluator, SequenctialEvaluator},
    thread_pool::ThreadPool,
};

use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::{borrow::Borrow, os::unix::thread, sync::Arc};

#[derive(Clone, Debug)]
pub struct ThreadSafePythonFn {
    func: Arc<Py<PyAny>>,
}

impl ThreadSafePythonFn {
    pub fn new(func: PyObject) -> Self {
        Self {
            func: Arc::new(func.into()),
        }
    }

    pub fn call<'py>(&self, outer: Python<'py>, input: AnyValue<'static>) -> Score {
        let func = self.func.clone();

        let av = outer.allow_threads(|| {
            Python::with_gil(|py| {
                let arg = Wrap(input);
                func.call1(py, (arg,)).expect("Python call failed")
            })
        });

        let av = py_object_to_any_value(&av.bind_borrowed(outer), true).unwrap();

        match av {
            AnyValue::Float32(score) => Score::from(score),
            AnyValue::Float64(score) => Score::from(score as f32),
            AnyValue::Int32(score) => Score::from(score as f32),
            AnyValue::Int64(score) => Score::from(score as f32),
            AnyValue::Int128(score) => Score::from(score as f32),
            AnyValue::Int16(score) => Score::from(score as f32),
            AnyValue::Int8(score) => Score::from(score as f32),
            AnyValue::Boolean(score) => Score::from(if score { 1.0 } else { 0.0 }),
            AnyValue::Null => Score::from(0.0),
            _ => panic!("Fitness function must return a number"),
        }
    }
}

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
    pub chromosome: Field,
    pub target: Field,
    pub epoch: Field,
    pub engine: Option<Box<dyn PyEngineTrait>>,
}

#[pymethods]
impl PyEngine {
    pub fn run(&mut self, generations: usize) {
        let maybe_engine = self.engine.take();
        if let Some(engine) = maybe_engine {
            let mut engine = engine
                .into_inner()
                .downcast::<GeneticEngine<FloatChromosome, Vec<Vec<f32>>>>()
                .unwrap();

            println!("Running engine for {} generations", generations);
            engine.run(|epoch| {
                println!("Generation: {}", epoch.index());
                epoch.index() < generations
            });
        }
    }
}

#[pymethods]
impl PyEngine {
    #[staticmethod]
    #[pyo3(signature = (num_genes, num_chromosomes, fitness_fn, range=None, bounds=None,
    survivor_selector=None, parent_selector=None, alters=None))]
    pub fn try_build_float_engine<'py>(
        num_genes: usize,
        num_chromosomes: usize,
        fitness_fn: PyObject,
        range: Option<(f32, f32)>,
        bounds: Option<(f32, f32)>,
        survivor_selector: Option<PyObject>,
        parent_selector: Option<PyObject>,
        alters: Option<PyObject>,
    ) -> PyResult<PyEngine> {
        let fitness_fn = ThreadSafePythonFn::new(fitness_fn);

        let range = range.map(|(min, max)| min..max).unwrap_or(0.0..1.0);
        let bounds = bounds.map(|(min, max)| min..max).unwrap_or(range.clone());

        let codex = FloatCodex::matrix(num_chromosomes, num_genes, range).with_bounds(bounds);

        let engine = GeneticEngine::builder()
            .codex(codex)
            .evaluator(SequenctialEvaluator)
            .fitness_fn(move |decoded: Vec<Vec<f32>>| {
                Python::with_gil(|py| {
                    let wrapped_decoded = AnyValue::from(decoded);
                    let result = fitness_fn.call(py, wrapped_decoded);

                    result
                })
            })
            .build();

        let chromosome_field = Field::new(
            std::any::type_name::<FloatChromosome>().to_string(),
            DataType::Struct(vec![
                Field::new("allele".to_string(), DataType::Float32),
                Field::new("fitness".to_string(), DataType::Float32),
            ]),
        );

        let target_field = Field::new(
            std::any::type_name::<Vec<Vec<f32>>>().to_string(),
            DataType::List(Box::new(Field::new(
                "target".to_string(),
                DataType::Float32,
            ))),
        );
        let epoch_field = Field::new(std::any::type_name::<usize>().to_string(), DataType::Int32);

        Ok(PyEngine {
            chromosome: chromosome_field,
            target: target_field,
            epoch: epoch_field,
            engine: Some(Box::new(engine)),
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

pub struct PythonEvaluator(pub ThreadSafePythonFn);

impl<C, T> Evaluator<C, T> for PythonEvaluator
where
    C: Chromosome + 'static,
    T: Clone + Send + Sync + 'static,
    AnyValue<'static>: From<T>,
{
    fn eval(
        &self,
        ecosystem: &mut Ecosystem<C>,
        thread_pool: Arc<ThreadPool>,
        problem: Arc<dyn Problem<C, T>>,
    ) -> usize {
        let genotypes = ecosystem
            .population
            .iter_mut()
            .enumerate()
            .filter(|(_, individual)| individual.score().is_none())
            .map(|(idx, individual)| {
                let geno = individual.take_genotype();
                let decoded = problem.decode(&geno);
                (idx, geno, AnyValue::from(decoded))
            })
            .collect::<Vec<_>>();

        let mut work_results = Vec::new();
        for (idx, geno, new_problem) in genotypes {
            let prob = self.0.clone();
            let handle = thread_pool.submit_with_result(move || {
                Python::with_gil(|py| {
                    let score = prob.call(py, new_problem);
                    (idx, score, geno)
                })
                // Acquire GIL in thread for Python call
            });

            work_results.push(handle);
        }

        let count = work_results.len();
        for work_result in work_results {
            let (idx, score, genotype) = work_result.result();
            ecosystem.population[idx].set_score(Some(score));
            ecosystem.population[idx].set_genotype(genotype);
        }

        count
    }
}

// Python::with_gil(|py| {
//     let wrapped_decoded = Wrap(AnyValue::from(decoded));
//     let result = fitness_fn.call1(py, (wrapped_decoded,));

//     let fitness = match result {
//         Ok(value) => {
//             // let value = value.extract::<Wrap<AnyValue<'_>>>(py).unwrap();
//             // value.0
//             let borrowed_value = value.bind(py);
//             py_object_to_any_value(borrowed_value, true)
//                 .unwrap()
//                 .into_static()
//         }
//         Err(e) => panic!("Error evaluating fitness function: {}", e),
//     };
//     let score = match fitness {
//         AnyValue::Float32(score) => Score::from(score),
//         AnyValue::Float64(score) => Score::from(score as f32),
//         AnyValue::Int32(score) => Score::from(score as f32),
//         AnyValue::Int64(score) => Score::from(score as f32),
//         AnyValue::Int128(score) => Score::from(score as f32),
//         AnyValue::Int16(score) => Score::from(score as f32),
//         AnyValue::Int8(score) => Score::from(score as f32),
//         AnyValue::Boolean(score) => Score::from(if score { 1.0 } else { 0.0 }),
//         AnyValue::Null => Score::from(0.0),
//         _ => panic!("Fitness function must return a number"),
//     };
//     score

// })
