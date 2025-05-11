use crate::conversion::ObjectValue;
use pyo3::{
    Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{CharChromosome, Chromosome, FnCodex, Gene};

#[pyclass]
#[derive(Clone)]
pub struct PyCharCodex {
    pub codex: FnCodex<CharChromosome, ObjectValue>,
}

#[pymethods]
impl PyCharCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, char_set=None))]
    pub fn new(chromosome_lengths: Option<Vec<usize>>, char_set: Option<String>) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);

        PyCharCodex {
            codex: FnCodex::new()
                .with_encoder(move || {
                    lengths
                        .iter()
                        .map(|len| CharChromosome::from((*len, char_set.clone())))
                        .collect::<Vec<CharChromosome>>()
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

unsafe impl Send for PyCharCodex {}
unsafe impl Sync for PyCharCodex {}
