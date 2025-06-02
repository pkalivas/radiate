use super::PyCodec;
use crate::{ObjectValue, PyGenotype};
use pyo3::{
    pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{BitChromosome, Chromosome, Codec, Gene};

#[pyclass]
#[derive(Clone)]
pub struct PyBitCodec {
    pub codec: PyCodec<BitChromosome>,
}

#[pymethods]
impl PyBitCodec {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None))]
    pub fn new(chromosome_lengths: Option<Vec<usize>>) -> Self {
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

    pub fn py_encode(&self) -> PyGenotype {
        let encoded = self.codec.encode();
        PyGenotype::from(encoded)
    }
}

unsafe impl Send for PyBitCodec {}
unsafe impl Sync for PyBitCodec {}
