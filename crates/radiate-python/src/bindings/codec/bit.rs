use super::PyCodec;
use crate::{ObjectValue, PyGenotype};
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, pyclass, pymethods,
    types::{PyAnyMethods, PyList, PyListMethods},
};
use radiate::{BitChromosome, Chromosome, Codec, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyBitCodec {
    pub codec: PyCodec<BitChromosome, ObjectValue>,
}

#[pymethods]
impl PyBitCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    pub fn decode_py<'py>(
        &self,
        py: pyo3::Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        let genotype: Genotype<BitChromosome> = genotype.clone().into();
        let obj_value = self.codec.decode_with_py(py, &genotype);
        obj_value.into_bound_py_any(py)
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, use_numpy=false))]
    pub fn matrix(chromosome_lengths: Option<Vec<usize>>, use_numpy: bool) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let decoder_lengths = lengths.iter().map(|len| *len).collect::<Vec<usize>>();

        PyBitCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| BitChromosome::new(*len))
                        .collect::<Vec<BitChromosome>>()
                        .into()
                })
                .with_decoder(move |py, geno| {
                    if use_numpy {
                        let np = py.import("numpy").unwrap();
                        let values = geno
                            .iter()
                            .flat_map(|chrom| chrom.iter().map(|gene| *gene.allele()))
                            .collect::<Vec<bool>>();
                        let outer = np.getattr("array").unwrap().call1((values,)).unwrap();

                        if decoder_lengths.len() > 1 {
                            let reshaped = outer
                                .call_method1("reshape", (decoder_lengths.clone(),))
                                .unwrap();

                            ObjectValue {
                                inner: reshaped.unbind().into_any(),
                            }
                        } else {
                            ObjectValue {
                                inner: outer.unbind().into_any(),
                            }
                        }
                    } else {
                        let outer = PyList::empty(py);
                        for chromo in geno.iter() {
                            let inner = PyList::empty(py);
                            for gene in chromo.iter() {
                                inner.append(*gene.allele()).unwrap();
                            }
                            outer.append(inner).unwrap();
                        }

                        return ObjectValue {
                            inner: outer.unbind().into_any(),
                        };
                    }
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_length=1, use_numpy=false))]
    pub fn vector(chromosome_length: Option<usize>, use_numpy: bool) -> Self {
        let length = chromosome_length.unwrap_or(1);

        PyBitCodec {
            codec: PyCodec::new()
                .with_encoder(move || Genotype::from(vec![BitChromosome::new(length)]))
                .with_decoder(move |py, geno| {
                    if use_numpy {
                        let values = geno
                            .iter()
                            .flat_map(|chrom| chrom.iter().map(|gene| *gene.allele()))
                            .collect::<Vec<bool>>();

                        let np = py.import("numpy").unwrap();
                        let outer = np.getattr("array").unwrap().call1((values,)).unwrap();

                        return ObjectValue {
                            inner: outer.unbind().into_any(),
                        };
                    } else {
                        let outer = PyList::empty(py);
                        for chrom in geno.iter() {
                            for gene in chrom.iter() {
                                outer.append(*gene.allele()).unwrap();
                            }
                        }

                        return ObjectValue {
                            inner: outer.unbind().into_any(),
                        };
                    }
                }),
        }
    }
}

unsafe impl Send for PyBitCodec {}
unsafe impl Sync for PyBitCodec {}
