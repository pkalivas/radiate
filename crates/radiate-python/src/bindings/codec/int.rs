use super::PyCodec;
use crate::{PyAnyObject, PyChromosome, PyGene, PyGenotype};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, pyclass, pymethods, types::PyInt};
use radiate::{Chromosome, Codec, Gene, Genotype, IntChromosome, IntGene};

#[pyclass]
#[derive(Clone)]
pub struct PyIntCodec {
    pub codec: PyCodec<IntChromosome<i32>, PyAnyObject>,
}

#[pymethods]
impl PyIntCodec {
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
    #[pyo3(signature = (chromosomes, use_numpy=false))]
    pub fn from_chromosomes(chromosomes: Vec<PyChromosome>, use_numpy: bool) -> Self {
        PyIntCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(
                        chromosomes
                            .iter()
                            .map(|chrom| IntChromosome::from(chrom.clone()))
                            .collect::<Vec<IntChromosome<i32>>>(),
                    )
                })
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (genes, use_numpy=false))]
    pub fn from_genes(genes: Vec<PyGene>, use_numpy: bool) -> Self {
        PyIntCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    IntChromosome::from(
                        genes
                            .iter()
                            .map(|gene| IntGene::from(gene.clone()))
                            .collect::<Vec<IntGene<i32>>>(),
                    )
                    .into()
                })
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None, use_numpy=false))]
    pub fn matrix(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(i32, i32)>,
        bound_range: Option<(i32, i32)>,
        use_numpy: bool,
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
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
                }),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (length=1, value_range=None, bound_range=None, use_numpy=false))]
    pub fn vector(
        length: usize,
        value_range: Option<(i32, i32)>,
        bound_range: Option<(i32, i32)>,
        use_numpy: bool,
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
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
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

                    PyAnyObject {
                        inner: outer.unbind().into_any(),
                    }
                }),
        }
    }
}

unsafe impl Send for PyIntCodec {}
unsafe impl Sync for PyIntCodec {}
