use crate::{IntoPyAnyObject, PyAnyObject, PyFitnessInner, bindings::PyCodec};
use pyo3::{Py, PyAny, Python, types::PyList};
use radiate::{Chromosome, Codec, Genotype, Problem, Score, error};

pub struct PyProblem<C: Chromosome, T> {
    fitness_func: PyAnyObject,
    codec: PyCodec<C, T>,
    is_batch: bool,
}

impl<C: Chromosome, T> PyProblem<C, T> {
    pub fn new(fitness_func: Py<PyAny>, codec: PyCodec<C, T>, is_batch: bool) -> Self {
        PyProblem {
            fitness_func: PyAnyObject {
                inner: fitness_func,
            },
            codec,
            is_batch,
        }
    }

    fn call_fitness<'py>(&self, py: Python<'py>, phenotype: PyAnyObject) -> radiate::Result<Score> {
        let any_value = self.fitness_func.inner.call1(py, (phenotype.inner,)).map_err(|e| {
            error::radiate_err!(Evaluation:
                "Ensure the function is callable, accepts one argument (Genotype), and returns a valid score. Details: {}",
                e
            )
        })?;

        let score = if let Ok(parsed) = any_value.extract::<f32>(py) {
            Score::from(parsed)
        } else if let Ok(parsed) = any_value.extract::<i32>(py) {
            Score::from(parsed)
        } else if let Ok(parsed) = any_value.extract::<f64>(py) {
            Score::from(parsed)
        } else if let Ok(parsed) = any_value.extract::<i64>(py) {
            Score::from(parsed)
        } else if let Ok(scores_vec) = any_value.extract::<Vec<f32>>(py) {
            if scores_vec.is_empty() {
                error::radiate_bail!(Evaluation:
                    "Fitness function returned an empty score vector."
                );
            } else {
                Score::from(scores_vec)
            }
        } else if let Ok(scores_vec) = any_value.extract::<Vec<i32>>(py) {
            if scores_vec.is_empty() {
                error::radiate_bail!(Evaluation:
                    "Fitness function returned an empty score vector."
                );
            } else {
                Score::from(scores_vec)
            }
        } else {
            error::radiate_bail!(Evaluation:
                "Failed to extract fitness score from Python function call. Ensure the function returns a valid score type."
            );
        };

        Ok(score)
    }

    fn call_batch_fitness<'py>(
        &self,
        py: Python<'py>,
        phenotypes: Py<PyAny>,
    ) -> radiate::Result<Vec<Score>> {
        let any_value = self
            .fitness_func
            .inner
            .call1(py, (phenotypes,))
            .map_err(|e| {
                error::radiate_err!(Evaluation:
                    "Ensure the function is callable, accepts one argument (list of Genotypes), and returns a valid list of scores. Details: {}",
                    e
                )
            })?;

        let scores = if let Ok(vals) = any_value.extract::<Vec<f32>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else if let Ok(vals) = any_value.extract::<Vec<i32>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else if let Ok(vals) = any_value.extract::<Vec<f64>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else if let Ok(vals) = any_value.extract::<Vec<i64>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else if let Ok(vals) = any_value.extract::<Vec<Vec<f32>>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else if let Ok(vals) = any_value.extract::<Vec<Vec<i32>>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else if let Ok(vals) = any_value.extract::<Vec<Vec<f64>>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else if let Ok(vals) = any_value.extract::<Vec<Vec<i64>>>(py) {
            vals.into_iter().map(Score::from).collect()
        } else {
            error::radiate_bail!(Evaluation:
                "Fitness function did not return a valid list of scores."
            );
        };

        Ok(scores)
    }
}

impl<C: Chromosome, T: IntoPyAnyObject> Problem<C, T> for PyProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> radiate::Result<Score> {
        Python::attach(|py| {
            let phenotype = self.codec.decode_with_py(py, individual).into_py(py);
            self.call_fitness(py, phenotype)
        })
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> radiate::Result<Vec<Score>> {
        Python::attach(|py| {
            if !self.is_batch {
                individuals
                    .iter()
                    .map(|ind| {
                        let phenotype = self.codec.decode_with_py(py, ind).into_py(py);
                        self.call_fitness(py, phenotype)
                    })
                    .collect()
            } else {
                let phenotypes = PyList::new(
                    py,
                    individuals
                        .iter()
                        .map(|ind| self.codec.decode_with_py(py, ind).into_py(py))
                        .map(|p| p.inner),
                )?;

                self.call_batch_fitness(py, phenotypes.into())
            }
        })
    }
}

unsafe impl<C: Chromosome, T> Send for PyProblem<C, T> {}
unsafe impl<C: Chromosome, T> Sync for PyProblem<C, T> {}

impl<C, T> From<(PyFitnessInner, PyCodec<C, T>)> for PyProblem<C, T>
where
    C: Chromosome,
    T: IntoPyAnyObject,
{
    fn from(value: (PyFitnessInner, PyCodec<C, T>)) -> Self {
        let (fitness_fn, codec) = value;
        PyProblem::new(
            fitness_fn.get_fitness_fn().unwrap(),
            codec,
            fitness_fn.is_batch(),
        )
    }
}
