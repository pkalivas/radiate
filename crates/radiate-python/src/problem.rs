use crate::{
    AnyValue,
    codex::PyCodex,
    conversion::{ObjectValue, Wrap},
};
use pyo3::{Py, PyAny, PyObject, Python};
use radiate::{Chromosome, Codex, Genotype, Problem, Score};
use std::sync::Arc;

pub struct PyProblem<C: Chromosome> {
    pub fitness_func: ThreadSafePythonFn,
    pub codex: PyCodex<C>,
}

impl<C: Chromosome> PyProblem<C> {
    pub fn new(fitness_func: PyObject, codex: PyCodex<C>) -> Self {
        PyProblem {
            fitness_func: ThreadSafePythonFn::new(fitness_func),
            codex,
        }
    }
}

impl<C: Chromosome> Problem<C, ObjectValue> for PyProblem<C> {
    fn encode(&self) -> Genotype<C> {
        self.codex.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> ObjectValue {
        self.codex.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        Python::with_gil(|py| {
            let phenotype = self.codex.decode_with_py(py, individual);
            self.fitness_func.call(py, phenotype)
        })
    }
}

unsafe impl<C: Chromosome> Send for PyProblem<C> {}
unsafe impl<C: Chromosome> Sync for PyProblem<C> {}

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

    pub fn call<'py>(&self, outer: Python<'py>, input: ObjectValue) -> Score {
        let any_value = self
            .func
            .call1(outer, (input.inner.bind_borrowed(outer),))
            .expect("Python call failed");

        let av = any_value
            .extract::<Wrap<AnyValue<'_>>>(outer)
            .expect("Python function must return a valid value")
            .0;

        match av {
            AnyValue::Float32(score) => Score::from(score),
            AnyValue::Float64(score) => Score::from(score as f32),
            AnyValue::Int32(score) => Score::from(score),
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
