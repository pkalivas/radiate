use std::sync::Arc;

use crate::{IntoPyObjectValue, ObjectValue, bindings::PyCodec};
use pyo3::{Borrowed, PyAny, PyObject, Python};
use radiate::{
    Chromosome, Codec, FitnessFunction, FnCodec, Genotype, NoveltySearch, Problem, Score,
    problem::Novelty,
};

pub struct PyNoveltyProblem<C: Chromosome, T, BD: Novelty<T>> {
    pub codec: FnCodec<C, T>,
    pub decoder: Arc<dyn Fn(Python<'_>, &Genotype<C>) -> ObjectValue>,
    pub search: NoveltySearch<T, BD>,
}

impl<C: Chromosome, T, BD: Novelty<T>> PyNoveltyProblem<C, T, BD> {
    pub fn new(
        codec: FnCodec<C, T>,
        decoder: Arc<dyn Fn(Python<'_>, &Genotype<C>) -> ObjectValue>,
        search: NoveltySearch<T, BD>,
    ) -> Self {
        PyNoveltyProblem {
            codec,
            decoder,
            search,
        }
    }
}

impl<C: Chromosome, T, BD: Novelty<T>> Problem<C, ObjectValue> for PyNoveltyProblem<C, T, BD>
where
    T: Send + Sync + 'static,
{
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> ObjectValue {
        Python::with_gil(|py| (self.decoder)(py, genotype))
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        let phenotype = self.codec.decode(individual);
        Score::from(self.search.evaluate(phenotype))
    }
}

unsafe impl<C: Chromosome, T, BD: Novelty<T>> Send for PyNoveltyProblem<C, T, BD> {}
unsafe impl<C: Chromosome, T, BD: Novelty<T>> Sync for PyNoveltyProblem<C, T, BD> {}

pub struct PyProblem<C: Chromosome, T: IntoPyObjectValue> {
    fitness_func: PyObject,
    codec: PyCodec<C, T>,
}

impl<C: Chromosome, T: IntoPyObjectValue> PyProblem<C, T> {
    pub fn new(fitness_func: PyObject, codec: PyCodec<C, T>) -> Self {
        PyProblem {
            fitness_func,
            codec,
        }
    }

    pub fn fitness_func(&self) -> &PyObject {
        &self.fitness_func
    }

    pub fn decode_with_py<'py>(&self, py: Python<'py>, genotype: &Genotype<C>) -> T {
        self.codec.decode_with_py(py, genotype)
    }
}

impl<C: Chromosome, T: IntoPyObjectValue> Problem<C, T> for PyProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        Python::with_gil(|py| {
            let phenotype = self.codec.decode_with_py(py, individual).into_py(py);
            let fitness_func = self.fitness_func.bind_borrowed(py);
            call_fitness(py, fitness_func, phenotype.inner.bind_borrowed(py))
        })
    }
}

impl<C: Chromosome + Clone, T: IntoPyObjectValue + Clone> Clone for PyProblem<C, T> {
    fn clone(&self) -> Self {
        Python::with_gil(|py| {
            let fitness_func = self.fitness_func.clone_ref(py);
            let codec = self.codec.clone();
            PyProblem {
                fitness_func,
                codec,
            }
        })
    }
}

unsafe impl<C: Chromosome, T: IntoPyObjectValue> Send for PyProblem<C, T> {}
unsafe impl<C: Chromosome, T: IntoPyObjectValue> Sync for PyProblem<C, T> {}

pub(crate) fn call_fitness<'a, 'py>(
    py: Python<'py>,
    func: Borrowed<'a, 'py, PyAny>,
    input: Borrowed<'a, 'py, PyAny>,
) -> Score {
    let any_value = func
        .as_ref()
        .call1(py, (input,))
        .expect("Python call failed");

    if let Ok(parsed) = any_value.extract::<f32>(py) {
        return Score::from(parsed);
    } else if let Ok(parsed) = any_value.extract::<i32>(py) {
        return Score::from(parsed as f32);
    } else if let Ok(parsed) = any_value.extract::<f64>(py) {
        return Score::from(parsed as f32);
    } else if let Ok(parsed) = any_value.extract::<i64>(py) {
        return Score::from(parsed as f32);
    } else if let Ok(scores_vec) = any_value.extract::<Vec<f32>>(py) {
        if scores_vec.is_empty() {
            return Score::from(0.0);
        } else {
            return Score::from(scores_vec);
        }
    } else if let Ok(scores_vec) = any_value.extract::<Vec<i32>>(py) {
        if scores_vec.is_empty() {
            return Score::from(0.0);
        } else {
            return Score::from(scores_vec.into_iter().map(|s| s as f32).collect::<Vec<_>>());
        }
    }

    panic!(
        "Failed to extract scores from Python function call. Ensure the function returns a valid score type."
    );
}
