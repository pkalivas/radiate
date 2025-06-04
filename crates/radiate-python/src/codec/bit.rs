use super::PyCodec;
use crate::ObjectValue;
use pyo3::{
    pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{BitChromosome, Chromosome, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyBitCodec {
    pub codec: PyCodec<BitChromosome>,
}

#[pymethods]
impl PyBitCodec {
    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None))]
    pub fn matrix(chromosome_lengths: Option<Vec<usize>>) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);

        PyBitCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| BitChromosome::new(*len))
                        .collect::<Vec<BitChromosome>>()
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
    #[pyo3(signature = (chromosome_length=1))]
    pub fn vector(chromosome_length: Option<usize>) -> Self {
        let length = chromosome_length.unwrap_or(1);

        PyBitCodec {
            codec: PyCodec::new()
                .with_encoder(move || Genotype::from(vec![BitChromosome::new(length)]))
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

unsafe impl Send for PyBitCodec {}
unsafe impl Sync for PyBitCodec {}
