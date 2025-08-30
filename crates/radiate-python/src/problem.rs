use crate::{IntoPyAnyObject, PyAnyObject, bindings::PyCodec};
use pyo3::{Py, PyAny, Python};
use radiate::{Chromosome, Codec, Genotype, Problem, Score};

pub struct PyProblem<C: Chromosome, T> {
    fitness_func: PyAnyObject,
    codec: PyCodec<C, T>,
}

impl<C: Chromosome, T> PyProblem<C, T> {
    pub fn new(fitness_func: Py<PyAny>, codec: PyCodec<C, T>) -> Self {
        PyProblem {
            fitness_func: PyAnyObject {
                inner: fitness_func,
            },
            codec,
        }
    }
}

impl<C: Chromosome, T: IntoPyAnyObject> Problem<C, T> for PyProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        Python::attach(|py| {
            let phenotype = self.codec.decode_with_py(py, individual).into_py(py);
            call_fitness(py, self.fitness_func.clone(), phenotype)
        })
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> Vec<Score> {
        Python::attach(|py| {
            individuals
                .iter()
                .map(|ind| {
                    let phenotype = self.codec.decode_with_py(py, ind).into_py(py);
                    call_fitness(py, self.fitness_func.clone(), phenotype)
                })
                .collect()
        })
    }
}

unsafe impl<C: Chromosome, T> Send for PyProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for PyProblem<C, T> {}

pub(crate) fn call_fitness<'a, 'py>(
    py: Python<'py>,
    func: PyAnyObject,
    input: PyAnyObject,
) -> Score {
    let any_value = func
        .inner
        .call1(py, (input.inner,))
        .expect("Python call failed");

    if let Ok(parsed) = any_value.extract::<f32>(py) {
        return Score::from(parsed);
    } else if let Ok(parsed) = any_value.extract::<i32>(py) {
        return Score::from(parsed);
    } else if let Ok(parsed) = any_value.extract::<f64>(py) {
        return Score::from(parsed);
    } else if let Ok(parsed) = any_value.extract::<i64>(py) {
        return Score::from(parsed);
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
