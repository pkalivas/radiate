use crate::{IntoPyAnyObject, PyAnyObject, PyFitnessInner, bindings::PyCodec};
use numpy::{PyArrayDyn, PyArrayMethods};
use pyo3::{Bound, Py, PyAny, Python, types::PyList};
use radiate::{Chromosome, Codec, Genotype, Problem, RadiateResult, Score, error};

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

    fn call_fitness<'py>(&self, py: Python<'py>, phenotype: PyAnyObject) -> RadiateResult<Score> {
        let any_value = self.fitness_func.inner.call1(py, (phenotype.inner,)).map_err(|e| {
            error::radiate_err!(Evaluation:
                "Ensure the function is callable, accepts one argument (Genotype), and returns a valid score: {}",
                e
            )
        })?;

        let bound = any_value.bind(py);

        if let Some(score) = Self::score_from_numpy(bound) {
            return score;
        }
        if let Some(score) = Self::score_from_scalar(&any_value, py) {
            return Ok(score);
        }
        if let Some(score) = Self::score_from_vec(&any_value, py) {
            return score;
        }

        error::radiate_bail!(Evaluation:
            "Failed to extract fitness score from Python function call. Ensure the function returns a valid score type."
        );
    }

    fn score_from_numpy<'py>(bound: &Bound<'py, PyAny>) -> Option<RadiateResult<Score>> {
        if let Ok(array) = bound.cast::<PyArrayDyn<f32>>() {
            return Some(Self::numpy_f32_score(array));
        }
        if let Ok(array) = bound.cast::<PyArrayDyn<f64>>() {
            return Some(Self::numpy_f64_score(array));
        }
        None
    }

    fn numpy_f32_score(array: &Bound<PyArrayDyn<f32>>) -> RadiateResult<Score> {
        let readonly_view = array.readonly();
        let slice = readonly_view.as_slice().map_err(|e| {
            error::radiate_err!(Evaluation:
                "Fitness function returned a non-contiguous numpy array: {}", e
            )
        })?;
        if slice.is_empty() {
            error::radiate_bail!(Evaluation:
                "Fitness function returned an empty score array."
            );
        }
        Ok(Score::from(slice.to_vec()))
    }

    fn numpy_f64_score(array: &Bound<PyArrayDyn<f64>>) -> RadiateResult<Score> {
        let readonly_view = array.readonly();
        let slice = readonly_view.as_slice().map_err(|e| {
            error::radiate_err!(Evaluation:
                "Fitness function returned a non-contiguous numpy array: {}", e
            )
        })?;
        if slice.is_empty() {
            error::radiate_bail!(Evaluation:
                "Fitness function returned an empty score array."
            );
        }
        Ok(Score::from(
            slice.iter().map(|&x| x as f32).collect::<Vec<f32>>(),
        ))
    }

    fn score_from_scalar(any_value: &Py<PyAny>, py: Python) -> Option<Score> {
        if let Ok(parsed) = any_value.extract::<f32>(py) {
            return Some(Score::from(parsed));
        }
        if let Ok(parsed) = any_value.extract::<i32>(py) {
            return Some(Score::from(parsed));
        }
        if let Ok(parsed) = any_value.extract::<f64>(py) {
            return Some(Score::from(parsed));
        }
        if let Ok(parsed) = any_value.extract::<i64>(py) {
            return Some(Score::from(parsed));
        }
        None
    }

    fn score_from_vec(any_value: &Py<PyAny>, py: Python) -> Option<RadiateResult<Score>> {
        if let Ok(vals) = any_value.extract::<Vec<f32>>(py) {
            return Some(Self::vec_to_score(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<f64>>(py) {
            return Some(Self::vec_to_score(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<i32>>(py) {
            return Some(Self::vec_to_score(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<i64>>(py) {
            return Some(Self::vec_to_score(vals));
        }
        None
    }

    fn vec_to_score<V>(values: Vec<V>) -> RadiateResult<Score>
    where
        Score: From<Vec<V>>,
    {
        if values.is_empty() {
            error::radiate_bail!(Evaluation:
                "Fitness function returned an empty score vector."
            );
        }
        Ok(Score::from(values))
    }

    fn call_batch_fitness<'py>(
        &self,
        py: Python<'py>,
        phenotypes: Py<PyAny>,
    ) -> RadiateResult<Vec<Score>> {
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

        if let Ok(vals) = any_value.extract::<Vec<f32>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<i32>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<f64>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<i64>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<Vec<f32>>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<Vec<i32>>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<Vec<f64>>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }
        if let Ok(vals) = any_value.extract::<Vec<Vec<i64>>>(py) {
            return Ok(Self::scores_from_vec(vals));
        }

        error::radiate_bail!(Evaluation:
            "Fitness function did not return a valid list of scores."
        );
    }

    fn scores_from_vec<V>(values: Vec<V>) -> Vec<Score>
    where
        Score: From<V>,
    {
        values.into_iter().map(Score::from).collect()
    }
}

impl<C: Chromosome, T: IntoPyAnyObject> Problem<C, T> for PyProblem<C, T> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> RadiateResult<Score> {
        Python::attach(|py| {
            let phenotype = self.codec.decode_with_py(py, individual).into_py(py);
            self.call_fitness(py, phenotype)
        })
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> RadiateResult<Vec<Score>> {
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
