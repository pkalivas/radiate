use crate::{AnyChromosome, Wrap, py_object_to_any_value};
use pyo3::{IntoPyObjectExt, Py, PyAny, Python};
use radiate::{AlterResult, Chromosome, Mutate, random_provider};
use std::sync::Arc;

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
                    if let Some(field) = allele.get_struct_value(&self.name) {
                        let new_field = self
                            .gene_mutator
                            .call1(py, (Wrap(field),))
                            .expect("mutation function should not fail");

                        let new_any_value =
                            py_object_to_any_value(&new_field.into_bound_py_any(py).unwrap(), true)
                                .unwrap()
                                .into_static();

                        crate::value::set_struct_value_at_field(allele, &self.name, new_any_value);

                        count += 1;
                    }
                }
            }
        });

        count.into()
    }
}
