use crate::conversion::ObjectValue;
use pyo3::{
    Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{BitChromosome, Chromosome, FnCodex, Gene};

#[pyclass]
#[derive(Clone)]
pub struct PyBitCodex {
    pub codex: FnCodex<BitChromosome, ObjectValue>,
}

#[pymethods]
impl PyBitCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None))]
    pub fn new(chromosome_lengths: Option<Vec<usize>>) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);

        PyBitCodex {
            codex: FnCodex::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| BitChromosome::new(*len))
                        .collect::<Vec<BitChromosome>>()
                        .into()
                })
                .with_decoder(|geno| {
                    Python::with_gil(|py| {
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
                    })
                }),
        }
    }
}

unsafe impl Send for PyBitCodex {}
unsafe impl Sync for PyBitCodex {}
