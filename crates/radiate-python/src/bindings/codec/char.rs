use super::PyCodec;
use crate::{PyAnyObject, PyChromosome, PyGene, PyGenotype};
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyResult, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{CharChromosome, CharGene, Chromosome, Codec, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyCharCodec {
    pub codec: PyCodec<CharChromosome, PyAnyObject>,
}

#[pymethods]
impl PyCharCodec {
    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    pub fn decode_py<'py>(
        &self,
        py: pyo3::Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.codec
            .decode_with_py(py, &genotype.clone().into())
            .into_bound_py_any(py)
    }

    #[staticmethod]
    #[pyo3(signature = (chromosomes))]
    pub fn from_chromosomes(chromosomes: Vec<PyChromosome>) -> Self {
        PyCharCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(
                        chromosomes
                            .iter()
                            .map(|chrom| CharChromosome::from(chrom.clone()))
                            .collect::<Vec<CharChromosome>>(),
                    )
                })
                .with_decoder(move |py, geno| {
                    let values = geno
                        .iter()
                        .map(|chrom| chrom.iter().map(|gene| *gene.allele()).collect())
                        .collect::<Vec<Vec<char>>>();

                    let outer = PyList::empty(py);
                    for value in values {
                        let inner = PyList::empty(py);
                        for gene in value {
                            inner.append(gene).unwrap();
                        }
                        outer.append(inner).unwrap();
                    }

                    PyAnyObject {
                        inner: outer.unbind().into_any(),
                    }
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (genes))]
    pub fn from_genes(genes: Vec<PyGene>) -> Self {
        PyCharCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    CharChromosome::from(
                        genes
                            .iter()
                            .map(|gene| CharGene::from(gene.clone()))
                            .collect::<Vec<CharGene>>(),
                    )
                    .into()
                })
                .with_decoder(move |py, geno| {
                    let values: Vec<char> = geno
                        .iter()
                        .flat_map(|chrom| chrom.iter().map(|gene| *gene.allele()))
                        .collect();

                    let outer = PyList::empty(py);
                    for value in values {
                        outer.append(value).unwrap();
                    }

                    return PyAnyObject {
                        inner: outer.unbind().into_any(),
                    };
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, char_set=None))]
    pub fn matrix(chromosome_lengths: Option<Vec<usize>>, char_set: Option<String>) -> Self {
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
                .with_decoder(move |py, geno| {
                    let outer = PyList::empty(py);
                    for chromo in geno.iter() {
                        let inner = PyList::empty(py);
                        for gene in chromo.iter() {
                            inner.append(*gene.allele()).unwrap();
                        }
                        outer.append(inner).unwrap();
                    }

                    return PyAnyObject {
                        inner: outer.unbind().into_any(),
                    };
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (length=1, char_set=None))]
    pub fn vector(length: usize, char_set: Option<String>) -> Self {
        PyCharCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(vec![CharChromosome::from((length, char_set.clone()))])
                })
                .with_decoder(move |py, geno| {
                    let outer = PyList::empty(py);
                    for chrom in geno.iter() {
                        for gene in chrom.iter() {
                            outer.append(*gene.allele()).unwrap();
                        }
                    }

                    return PyAnyObject {
                        inner: outer.unbind().into_any(),
                    };
                }),
        }
    }
}

unsafe impl Send for PyCharCodec {}
unsafe impl Sync for PyCharCodec {}
