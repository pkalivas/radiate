use super::PyCodec;
use crate::ObjectValue;
use pyo3::{
    pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{CharChromosome, Chromosome, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyCharCodec {
    pub codec: PyCodec<CharChromosome>,
}

#[pymethods]
impl PyCharCodec {
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
