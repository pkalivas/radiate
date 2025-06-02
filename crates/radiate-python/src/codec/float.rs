use super::PyCodec;
use crate::ObjectValue;
use pyo3::{
    pyclass, pymethods,
    types::{PyFloat, PyList, PyListMethods},
};
use radiate::{Chromosome, FloatChromosome, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyFloatCodec {
    pub codec: PyCodec<FloatChromosome>,
}

#[pymethods]
impl PyFloatCodec {
    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None))]
    pub fn matrix(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
    ) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0.0..1.0);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

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
    #[pyo3(signature = (length=1, value_range=None, bound_range=None))]
    pub fn vector(
        length: usize,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
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
