// use crate::{ObjectValue, codec::PyCodec};
// use pyo3::{
//     Borrowed, PyAny, PyObject, Python,
//     sync::GILOnceCell,
//     types::{PyAnyMethods, PyFloat, PyInt, PyList},
// };
// use radiate::{Chromosome, Codec, Genotype, Problem, Score};

// static FITNESS_FN_CELL: GILOnceCell<PyObject> = GILOnceCell::new();

// pub struct PyProblem<C: Chromosome> {
//     pub fitness_func: PyObject,
//     pub codec: PyCodec<C>,
// }

// impl<C: Chromosome> PyProblem<C> {
//     pub fn new(fitness_func: PyObject, codec: PyCodec<C>) -> Self {
//         // Store the fitness function in a GILOnceCell to ensure it is only bound once
//         PyProblem {
//             fitness_func: fitness_func,
//             codec,
//         }
//     }
// }

// impl<C: Chromosome> Problem<C, ObjectValue> for PyProblem<C> {
//     fn encode(&self) -> Genotype<C> {
//         self.codec.encode()
//     }

//     fn decode(&self, genotype: &Genotype<C>) -> ObjectValue {
//         self.codec.decode(genotype)
//     }

//     fn eval(&self, individual: &Genotype<C>) -> Score {
//         Python::with_gil(|py| {
//             let phenotype = self.codec.decode_with_py(py, individual);
//             let func = self.fitness_func.bind_borrowed(py);

//             call(py, &func, phenotype)
//         })
//     }

//     fn eval_batch(&self, individuals: &[Genotype<C>]) -> Vec<Score> {
//         Python::with_gil(|py| {
//             let func = self.fitness_func.bind_borrowed(py);
//             individuals
//                 .iter()
//                 .map(|ind| {
//                     let phenotype = self.codec.decode_with_py(py, ind);
//                     call(py, &func, phenotype)
//                 })
//                 .collect()
//         })
//     }
// }

// unsafe impl<C: Chromosome> Send for PyProblem<C> {}
// unsafe impl<C: Chromosome> Sync for PyProblem<C> {}

// pub fn call<'py, 'a>(
//     py: Python<'py>,
//     func: &Borrowed<'py, 'a, PyAny>,
//     input: ObjectValue,
// ) -> Score {
//     let any_value = func
//         .call1((input.inner.bind_borrowed(py),))
//         .expect("Python call failed");

//     let temp = any_value;

//     if temp.is_instance_of::<PyFloat>() {
//         return Score::from(temp.extract::<f64>().expect("Expected a float") as f32);
//     } else if temp.is_instance_of::<PyInt>() {
//         if let Ok(score) = temp.extract::<i64>() {
//             return Score::from(score as f32);
//         }
//     } else if temp.is_instance_of::<PyList>() {
//         let list = temp.downcast::<PyList>().expect("Expected a list");
//         let mut score = Vec::new();
//         if let Ok(iter) = list.try_iter() {
//             for item in iter {
//                 let it = item.expect("Expected an item in the list");
//                 if it.is_instance_of::<PyFloat>() {
//                     if let Ok(value) = it.extract::<f64>() {
//                         score.push(value as f32);
//                     }
//                 } else if it.is_instance_of::<PyInt>() {
//                     if let Ok(value) = it.extract::<i64>() {
//                         score.push(value as f32);
//                     }
//                 }
//             }
//         }

//         if !score.is_empty() {
//             return Score::from(score);
//         }
//     }

//     panic!(
//         "Fitness function must return a float value, got: {:?}",
//         temp
//     );
// }

use crate::{ObjectValue, codec::PyCodec};
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
    let any_value = func
        .call1(py, (input.inner.bind_borrowed(py),))
        .expect("Python call failed");

    let temp = any_value.bind_borrowed(py);

    if temp.is_instance_of::<PyFloat>() {
        return Score::from(temp.extract::<f64>().expect("Expected a float") as f32);
    } else if temp.is_instance_of::<PyInt>() {
        if let Ok(score) = temp.extract::<i64>() {
            return Score::from(score as f32);
        }
    } else if temp.is_instance_of::<PyList>() {
        let list = temp.downcast::<PyList>().expect("Expected a list");
        let mut score = Vec::new();
        if let Ok(iter) = list.try_iter() {
            for item in iter {
                let it = item.expect("Expected an item in the list");
                if it.is_instance_of::<PyFloat>() {
                    if let Ok(value) = it.extract::<f64>() {
                        score.push(value as f32);
                    }
                } else if it.is_instance_of::<PyInt>() {
                    if let Ok(value) = it.extract::<i64>() {
                        score.push(value as f32);
                    }
                }
            }
        }

        if !score.is_empty() {
            return Score::from(score);
        }
    }

    panic!(
        "Fitness function must return a float value, got: {:?}",
        temp
    );
}
