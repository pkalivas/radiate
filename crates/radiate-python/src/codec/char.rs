use super::PyCodec;
use crate::{ObjectValue, PyGenotype};
use pyo3::{
    pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{CharChromosome, Chromosome, Codec, Gene};

#[pyclass]
#[derive(Clone)]
pub struct PyCharCodec {
    pub codec: PyCodec<CharChromosome>,
}

#[pymethods]
impl PyCharCodec {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, char_set=None))]
    pub fn new(chromosome_lengths: Option<Vec<usize>>, char_set: Option<String>) -> Self {
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

    pub fn py_encode(&self) -> PyGenotype {
        let encoded = self.codec.encode();
        PyGenotype::from(encoded)
    }
}

unsafe impl Send for PyCharCodec {}
unsafe impl Sync for PyCharCodec {}
