use crate::{ObjectValue, codec::PyCodec};
use pyo3::{Py, PyAny, PyObject, Python, sync::GILOnceCell};
use radiate::{Chromosome, Codec, Genotype, Problem, Score};

pub struct PyProblem<C: Chromosome> {
    fitness_fn_cell: GILOnceCell<PyObject>,
    codec: PyCodec<C>,
}

impl<C: Chromosome> PyProblem<C> {
    pub fn new(fitness_func: PyObject, codec: PyCodec<C>) -> Self {
        let cell = Python::with_gil(|py| {
            let cell = GILOnceCell::new();
            cell.set(py, fitness_func)
                .expect("Failed to set fitness function in GILOnceCell");
            cell
        });

        PyProblem {
            fitness_fn_cell: cell,
            codec,
        }
    }
}

impl<C: Chromosome> Problem<C, ObjectValue> for PyProblem<C> {
    fn encode(&self) -> Genotype<C> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<C>) -> ObjectValue {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<C>) -> Score {
        Python::with_gil(|py| {
            let func = self.fitness_fn_cell.get(py).unwrap();
            let phenotype = self.codec.decode_with_py(py, individual);
            call(py, &func, &phenotype.inner)
        })
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> Vec<Score> {
        Python::with_gil(|py| {
            let func = self.fitness_fn_cell.get(py).unwrap();

            individuals
                .iter()
                .map(|ind| self.codec.decode_with_py(py, ind).inner)
                .map(|phenotype| call(py, &func, &phenotype))
                .collect::<Vec<Score>>()
        })
    }
}

unsafe impl<C: Chromosome> Send for PyProblem<C> {}
unsafe impl<C: Chromosome> Sync for PyProblem<C> {}

pub fn call<'py>(py: Python<'py>, func: &Py<PyAny>, input: &Py<PyAny>) -> Score {
    let any_value = func.call1(py, (input,)).expect("Python call failed");

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
