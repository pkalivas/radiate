use crate::PyChromosome;
use pyo3::{Py, PyAny, Python};
use radiate::{AlterResult, Chromosome, Mutate, random_provider};
use std::sync::Arc;

pub struct PyMutator<C: Chromosome> {
    pub rate: f32,
    pub name: String,
    pub chromosome_mutator: Arc<Py<PyAny>>,
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome> PyMutator<C> {
    pub fn new(rate: f32, name: String, mutate_func: Py<PyAny>) -> Self {
        PyMutator {
            rate,
            name,
            chromosome_mutator: Arc::new(mutate_func),
            _marker: std::marker::PhantomData,
        }
    }
}

unsafe impl<C: Chromosome> Send for PyMutator<C> {}
unsafe impl<C: Chromosome> Sync for PyMutator<C> {}

impl<C> Mutate<C> for PyMutator<C>
where
    C: Chromosome + Clone + From<PyChromosome>,
    PyChromosome: From<C>,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;

        if random_provider::random::<f32>() < rate {
            Python::with_gil(|py| {
                let py_chromosome = PyChromosome::from(chromosome.clone());
                let result = self
                    .chromosome_mutator
                    .as_ref()
                    .call1(py, (py_chromosome,))
                    .expect("mutation function should not fail");
                let mutated = result
                    .extract::<PyChromosome>(py)
                    .expect("should return a PyChromosome");
                *chromosome = mutated.into();
            });

            count += 1;
        }

        count.into()
    }
}
