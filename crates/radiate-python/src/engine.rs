use crate::{
    AnyValue, DataType, Field,
    conversion::{Wrap, py_object_to_any_value},
};
use pyo3::{PyObject, PyResult, Python, pyclass, pymethods};
use radiate::{EngineExt, Epoch, FloatChromosome, FloatCodex, GeneticEngine, Score};

use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::sync::Arc;

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

    pub fn call(&self, input: AnyValue<'static>) -> Score {
        let func = self.func.clone();
        Python::with_gil(|py| {
            let arg = Wrap(input);
            let value = func.call1(py, (arg,)).expect("Python call failed");
            let av = py_object_to_any_value(&value.bind(py), true)
                .unwrap()
                .into_static();
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
        })
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
            // Python::with_gil(|py| unsafe {
            //     py.allow_threads(|| unsafe {
            //         engine.run(|epoch| {
            //             println!("Generation: {}", epoch.index());
            //             epoch.index() < generations
            //         })
            //     });
            // });

            // let result = engine
            //     .iter()
            //     .inspect(|ctx| log_ctx!(ctx))
            //     .take(generations)
            //     .last()
            //     .unwrap();
        }
        // if let Some(engine) = self.engine.as_mut() {}
        // Python::with_gil(|py| {
        //     let mut engine = self.engine.as_mut();
        //     for _ in 0..generations {
        //         engine.next();
        //     }
        //     let best = engine.best().unwrap();
        //     let best = best
        //         .iter()
        //         .map(|chromosome| {
        //             Wrap(AnyValue::from(
        //                 chromosome
        //                     .as_ref()
        //                     .iter()
        //                     .map(|gene| gene.allele().clone())
        //                     .collect::<Vec<_>>(),
        //             ))
        //         })
        //         .collect::<Vec<_>>();
        //     let best = Wrap(AnyValue::from(best));
        //     best.into_pyobject(py).unwrap()
        // });
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
            .fitness_fn(move |decoded: Vec<Vec<f32>>| {
                fitness_fn.call(AnyValue::from(decoded))

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
