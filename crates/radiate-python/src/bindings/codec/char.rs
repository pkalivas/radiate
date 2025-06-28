use super::PyCodec;
use crate::{ObjectValue, PyGenotype};
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{CharChromosome, Chromosome, Codec, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyCharCodec {
    pub codec: PyCodec<CharChromosome, ObjectValue>,
}

#[pymethods]
impl PyCharCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    pub fn decode_py<'py>(
        &self,
        py: pyo3::Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        let genotype: Genotype<CharChromosome> = genotype.clone().into();
        let obj_value = self.codec.decode_with_py(py, &genotype);
        obj_value.into_bound_py_any(py)
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, char_set=None))]
    pub fn matrix(chromosome_lengths: Option<Vec<usize>>, char_set: Option<String>) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);

        PyCharCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| CharChromosome::from((*len, char_set.clone())))
                        .collect::<Vec<CharChromosome>>()
                        .into()
                })
                .with_decoder(|py, geno| {
                    let outer = PyList::empty(py);
                    for chromo in geno.iter() {
                        let inner = PyList::empty(py);
                        for gene in chromo.iter() {
                            inner.append(*gene.allele()).unwrap();
                        }
                        outer.append(inner).unwrap();
                    }

                    ObjectValue {
                        inner: outer.unbind().into_any(),
                    }
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (length=1, char_set=None))]
    pub fn vector(length: usize, char_set: Option<String>) -> Self {
        PyCharCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(vec![CharChromosome::from((length, char_set.clone()))])
                })
                .with_decoder(|py, geno| {
                    let outer = PyList::empty(py);
                    for chrom in geno.iter() {
                        for gene in chrom.iter() {
                            outer.append(*gene.allele()).unwrap();
                        }
                    }

                    ObjectValue {
                        inner: outer.unbind().into_any(),
                    }
                }),
        }
    }
}

unsafe impl Send for PyCharCodec {}
unsafe impl Sync for PyCharCodec {}
