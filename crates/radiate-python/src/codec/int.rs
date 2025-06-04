use super::PyCodec;
use crate::ObjectValue;
use pyo3::{
    pyclass, pymethods,
    types::{PyInt, PyList, PyListMethods},
};
use radiate::{Chromosome, Gene, IntChromosome};

#[pyclass]
#[derive(Clone)]
pub struct PyIntCodec {
    pub codec: PyCodec<IntChromosome<i32>>,
}

#[pymethods]
impl PyIntCodec {
    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None))]
    pub fn matrix(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(i32, i32)>,
        bound_range: Option<(i32, i32)>,
    ) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0..1);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        PyIntCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| {
                            IntChromosome::from((*len, val_range.clone(), bound_range.clone()))
                        })
                        .collect::<Vec<IntChromosome<i32>>>()
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
        value_range: Option<(i32, i32)>,
        bound_range: Option<(i32, i32)>,
    ) -> Self {
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0..1);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        PyIntCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    vec![IntChromosome::from((
                        length,
                        val_range.clone(),
                        bound_range.clone(),
                    ))]
                    .into()
                })
                .with_decoder(|py, geno| {
                    let values = geno
                        .iter()
                        .next()
                        .map(|chrom| {
                            chrom
                                .iter()
                                .map(|gene| *gene.allele() as i64)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let outer = PyList::new(py, values).unwrap();

                    ObjectValue {
                        inner: outer.unbind().into_any(),
                    }
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (value_range=None, bound_range=None))]
    pub fn scalar(value_range: Option<(i32, i32)>, bound_range: Option<(i32, i32)>) -> Self {
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0..1);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        PyIntCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    vec![IntChromosome::from((
                        1,
                        val_range.clone(),
                        bound_range.clone(),
                    ))]
                    .into()
                })
                .with_decoder(|py, geno| {
                    let val = geno
                        .iter()
                        .next()
                        .and_then(|chrom| chrom.iter().next())
                        .map_or(0, |gene| *gene.allele());
                    let outer = PyInt::new(py, val as i64);

                    ObjectValue {
                        inner: outer.unbind().into_any(),
                    }
                }),
        }
    }
}

unsafe impl Send for PyIntCodec {}
unsafe impl Sync for PyIntCodec {}
