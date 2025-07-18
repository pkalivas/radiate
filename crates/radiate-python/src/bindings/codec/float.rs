use super::PyCodec;
use crate::{ObjectValue, PyGenotype};
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, pyclass, pymethods,
    types::{PyAnyMethods, PyFloat, PyList, PyListMethods},
};
use radiate::{Chromosome, Codec, FloatChromosome, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyFloatCodec {
    pub codec: PyCodec<FloatChromosome, ObjectValue>,
}

#[pymethods]
impl PyFloatCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    pub fn decode_py<'py>(
        &self,
        py: pyo3::Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        let genotype: Genotype<FloatChromosome> = genotype.clone().into();
        let obj_value = self.codec.decode_with_py(py, &genotype);
        obj_value.into_bound_py_any(py)
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None, use_numpy=false))]
    pub fn matrix(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
        use_numpy: bool,
    ) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0.0..1.0);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        let decoder_lengths = lengths.iter().map(|len| *len).collect::<Vec<usize>>();
        PyFloatCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| {
                            FloatChromosome::from((*len, val_range.clone(), bound_range.clone()))
                        })
                        .collect::<Vec<FloatChromosome>>()
                        .into()
                })
                .with_decoder(move |py, geno| {
                    if use_numpy {
                        let np = py.import("numpy").unwrap();
                        let values: Vec<f32> = geno
                            .iter()
                            .flat_map(|chrom| chrom.iter().map(|gene| *gene.allele()))
                            .collect();
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
    #[pyo3(signature = (length=1, value_range=None, bound_range=None, use_numpy=false))]
    pub fn vector(
        length: usize,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
        use_numpy: bool,
    ) -> Self {
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0.0..1.0);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        PyFloatCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(vec![FloatChromosome::from((
                        length,
                        val_range.clone(),
                        bound_range.clone(),
                    ))])
                })
                .with_decoder(move |py, geno| {
                    if use_numpy {
                        let values: Vec<f32> = geno
                            .iter()
                            .flat_map(|chrom| chrom.iter().map(|gene| *gene.allele()))
                            .collect();

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

    #[staticmethod]
    #[pyo3(signature = (value_range=None, bound_range=None))]
    pub fn scalar(value_range: Option<(f32, f32)>, bound_range: Option<(f32, f32)>) -> Self {
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0.0..1.0);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        PyFloatCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(vec![FloatChromosome::from((
                        1,
                        val_range.clone(),
                        bound_range.clone(),
                    ))])
                })
                .with_decoder(|py, geno| {
                    let val = geno
                        .iter()
                        .next()
                        .and_then(|chrom| chrom.iter().next())
                        .map_or(0.0, |gene| *gene.allele());
                    let outer = PyFloat::new(py, val as f64);

                    ObjectValue {
                        inner: outer.unbind().into_any(),
                    }
                }),
        }
    }
}

unsafe impl Send for PyFloatCodec {}
unsafe impl Sync for PyFloatCodec {}
