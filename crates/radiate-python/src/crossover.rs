use crate::PyChromosome;
use pyo3::{Py, PyAny, Python};
use radiate::{AlterResult, Chromosome, Crossover, random_provider};
use std::sync::Arc;

pub struct PyCrossover<C: Chromosome> {
    pub rate: f32,
    pub name: String,
    pub chromosome_crossover: Arc<Py<PyAny>>,
    _marker: std::marker::PhantomData<C>,
}

impl<C: Chromosome> PyCrossover<C> {
    pub fn new(rate: f32, name: String, crossover_func: Py<PyAny>) -> Self {
        PyCrossover {
            rate,
            name,
            chromosome_crossover: Arc::new(crossover_func),
            _marker: std::marker::PhantomData,
        }
    }
}

unsafe impl<C: Chromosome> Send for PyCrossover<C> {}
unsafe impl<C: Chromosome> Sync for PyCrossover<C> {}

impl<C> Crossover<C> for PyCrossover<C>
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

    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;
        if random_provider::random::<f32>() < rate {
            Python::with_gil(|py| {
                let py_chrom_one = PyChromosome::from(chrom_one.clone());
                let py_chrom_two = PyChromosome::from(chrom_two.clone());
                let result = self
                    .chromosome_crossover
                    .as_ref()
                    .call1(py, (py_chrom_one, py_chrom_two))
                    .expect("crossover function should not fail");
                let (mutated_one, mutated_two) = result
                    .extract::<(PyChromosome, PyChromosome)>(py)
                    .expect("should return a tuple of PyChromosome");
                *chrom_one = mutated_one.into();
                *chrom_two = mutated_two.into();

                count += 1;
            });
        }

        count.into()
    }
}
