use crate::{AnyChromosome, PyChromosome, Wrap, py_object_to_any_value};
use pyo3::{IntoPyObjectExt, Py, PyAny, Python};
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

impl<C> Mutate<C> for PyMutator<C>
where
    C: Chromosome + Clone + Send + Sync + From<PyChromosome>,
    PyChromosome: From<C>,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn mutate_chromosome(&self, chromosome: &mut C, _: f32) -> AlterResult {
        Python::attach(|py| {
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

        0.into()
    }
}

pub struct AnyGeneMutator {
    pub rate: f32,
    pub name: String,
    pub gene_mutator: Arc<Py<PyAny>>,
}

impl AnyGeneMutator {
    pub fn new(rate: f32, name: String, mutate_func: Py<PyAny>) -> Self {
        AnyGeneMutator {
            rate,
            name,
            gene_mutator: Arc::new(mutate_func),
        }
    }
}

impl Mutate<AnyChromosome<'_>> for AnyGeneMutator {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn mutate_chromosome(&self, chromosome: &mut AnyChromosome<'_>, rate: f32) -> AlterResult {
        let mut count = 0;

        Python::attach(|py| {
            for gene in chromosome.genes_mut() {
                if random_provider::random::<f32>() < rate {
                    let allele = gene.allele_mut();
                    if let Some(field) = allele.get_field(&self.name) {
                        let new_field = self
                            .gene_mutator
                            .call1(py, (Wrap(field),))
                            .expect("mutation function should not fail");

                        let new_any_value =
                            py_object_to_any_value(&new_field.into_bound_py_any(py).unwrap(), true)
                                .unwrap()
                                .into_static();

                        // crate::value::merge_any_values(allele, new_any_value.into_static());
                        crate::value::set_any_value_at_field(allele, &self.name, new_any_value);

                        count += 1;
                    }
                }
            }
        });

        count.into()
    }
}
