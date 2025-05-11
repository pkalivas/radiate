use crate::conversion::ObjectValue;
use pyo3::{
    Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{Chromosome, FloatChromosome, FnCodex, Gene};

#[pyclass]
#[derive(Clone)]
pub struct PyFloatCodex {
    pub codex: FnCodex<FloatChromosome, ObjectValue>,
}

#[pymethods]
impl PyFloatCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None))]
    pub fn new(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(f32, f32)>,
        bound_range: Option<(f32, f32)>,
    ) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let val_range = value_range.map(|rng| rng.0..rng.1).unwrap_or(0.0..1.0);
        let bound_range = bound_range
            .map(|rng| rng.0..rng.1)
            .unwrap_or(val_range.clone());

        PyFloatCodex {
            codex: FnCodex::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| {
                            FloatChromosome::from((*len, val_range.clone(), bound_range.clone()))
                        })
                        .collect::<Vec<FloatChromosome>>()
                        .into()
                })
                .with_decoder(|geno| {
                    let res = Python::with_gil(|py| {
                        let outer = PyList::empty(py);
                        for chromo in geno.iter() {
                            let inner = PyList::empty(py);
                            for gene in chromo.iter() {
                                inner.append(*gene.allele()).unwrap();
                            }
                            outer.append(inner).unwrap();
                        }

                        outer.unbind()
                    });

                    ObjectValue {
                        inner: res.into_any(),
                    }
                }),
        }
    }
}

unsafe impl Send for PyFloatCodex {}
unsafe impl Sync for PyFloatCodex {}
