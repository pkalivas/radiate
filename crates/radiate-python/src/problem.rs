use crate::{AnyValue, ObjectValue, codec::PyCodec, conversion::py_object_to_any_value};
use pyo3::{
    Py, PyAny, PyObject, Python,
    sync::GILOnceCell,
    types::{PyAnyMethods, PyFloat, PyInt, PyList},
};
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
            let phenotype = self.codec.decode_with_py(py, individual);
            let func = self.fitness_fn_cell.get(py).unwrap();

            call(py, func, phenotype)
        })
    }

    fn eval_batch(&self, individuals: &[Genotype<C>]) -> Vec<Score> {
        Python::with_gil(|py| {
            let func = self.fitness_fn_cell.get(py).unwrap();

            individuals
                .iter()
                .map(|ind| {
                    let phenotype = self.codec.decode_with_py(py, ind);
                    call(py, &func, phenotype)
                })
                .collect()
        })
    }
}

unsafe impl<C: Chromosome> Send for PyProblem<C> {}
unsafe impl<C: Chromosome> Sync for PyProblem<C> {}

pub fn call<'py, 'a>(py: Python<'py>, func: &Py<PyAny>, input: ObjectValue) -> Score {
    let any_value = func.call1(py, (input.inner,)).expect("Python call failed");

    let output = py_object_to_any_value(any_value.bind(py), true).unwrap();

    if let AnyValue::Float64(score) = output {
        return Score::from(score as f32);
    }
    if let AnyValue::Int64(score) = output {
        return Score::from(score as f32);
    }
    if let AnyValue::Vec(scores) = output {
        if scores.is_empty() {
            return Score::from(0.0);
        }
        return Score::from(
            scores
                .into_iter()
                .map(|s| match s {
                    AnyValue::Float64(f) => f as f32,
                    AnyValue::Int64(i) => i as f32,
                    _ => panic!("Expected a float or int in the list, got: {:?}", s),
                })
                .collect::<Vec<_>>(),
        );
    }

    panic!("Fitness function must return a float value, got");
}
