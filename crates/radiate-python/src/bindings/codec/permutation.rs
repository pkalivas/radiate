use super::PyCodec;
use crate::{ObjectValue, PyGenotype};
use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, pyclass, pymethods, types::PyList};
use radiate::{
    Chromosome, Codec, Gene, Genotype, PermutationChromosome, PermutationGene, random_provider,
};
use std::sync::Arc;

#[pyclass]
#[derive(Clone)]
pub struct PyPermutationCodec {
    pub codec: PyCodec<PermutationChromosome<usize>, ObjectValue>,
    pub alleles: Arc<Vec<usize>>,
}

#[pymethods]
impl PyPermutationCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    pub fn decode_py<'py>(
        &self,
        py: pyo3::Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        let genotype: Genotype<PermutationChromosome<usize>> = genotype.clone().into();
        let obj_value = self.codec.decode_with_py(py, &genotype);
        obj_value.into_bound_py_any(py)
    }

    #[new]
    #[pyo3(signature = (alleles))]
    pub fn new(alleles: Vec<Py<PyAny>>) -> Self {
        let indexed_alleles = Arc::new((0..alleles.len()).into_iter().collect::<Vec<usize>>());
        let arc_alleles = Arc::new(alleles);
        let allele_count = arc_alleles.len();

        PyPermutationCodec {
            alleles: Arc::clone(&indexed_alleles),
            codec: PyCodec::new()
                .with_encoder(move || {
                    PermutationChromosome::new(
                        random_provider::indexes(0..allele_count)
                            .iter()
                            .map(|i| PermutationGene::new(*i, Arc::clone(&indexed_alleles)))
                            .collect(),
                        Arc::clone(&indexed_alleles),
                    )
                    .into()
                })
                .with_decoder(move |py, geno| {
                    let values = geno
                        .iter()
                        .flat_map(|chromosome| {
                            chromosome.genes().iter().map(|gene| {
                                let index = gene.allele();
                                arc_alleles[*index].clone_ref(py)
                            })
                        })
                        .collect::<Vec<Py<PyAny>>>()
                        .into_iter()
                        .map(|py_any| py_any.into_bound(py))
                        .collect::<Vec<Bound<PyAny>>>();

                    ObjectValue {
                        inner: PyList::new(py, values).unwrap().unbind().into_any(),
                    }
                }),
        }
    }
}

unsafe impl Send for PyPermutationCodec {}
unsafe impl Sync for PyPermutationCodec {}
