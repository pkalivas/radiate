mod any;
mod bit;
mod char;
mod float;
mod graph;
mod int;
mod permutation;
mod tree;

use std::sync::Arc;

pub use any::PyAnyCodec;
pub use bit::PyBitCodec;
pub use char::PyCharCodec;
pub use float::PyFloatCodec;
pub use graph::PyGraphCodec;
pub use int::PyIntCodec;
pub use permutation::PyPermutationCodec;
pub use tree::PyTreeCodec;

use numpy::{Element, PyArray, PyArray1, PyArrayMethods};
use pyo3::Python;
use pyo3::{
    Bound, IntoPyObject, PyAny, PyResult,
    types::{PyList, PyListMethods},
};
use radiate::{Chromosome, Codec, Gene, Genotype};

#[derive(Clone)]
pub struct PyCodec<C: Chromosome, T> {
    encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    decoder: Option<Arc<dyn Fn(Python<'_>, &Genotype<C>) -> T>>,
}

impl<C: Chromosome, T> PyCodec<C, T> {
    pub fn new() -> Self {
        PyCodec {
            encoder: None,
            decoder: None,
        }
    }

    pub fn decode_with_py(&self, py: Python<'_>, genotype: &Genotype<C>) -> T {
        match &self.decoder {
            Some(decoder) => decoder(py, genotype),
            None => panic!("Decoder function is not set"),
        }
    }

    pub fn with_encoder<F>(mut self, encoder: F) -> Self
    where
        F: Fn() -> Genotype<C> + 'static,
    {
        self.encoder = Some(Arc::new(encoder));
        self
    }

    pub fn with_decoder<F>(mut self, decoder: F) -> Self
    where
        F: Fn(Python<'_>, &Genotype<C>) -> T + 'static,
    {
        self.decoder = Some(Arc::new(decoder));
        self
    }
}

impl<C: Chromosome, T> Codec<C, T> for PyCodec<C, T> {
    fn encode(&self) -> Genotype<C> {
        match &self.encoder {
            Some(encoder) => encoder(),
            None => panic!("Encoder function is not set"),
        }
    }

    fn decode(&self, genotype: &Genotype<C>) -> T {
        Python::attach(|py| match &self.decoder {
            Some(decoder) => decoder(py, genotype),
            None => panic!("Decoder function is not set"),
        })
    }
}

unsafe impl<C: Chromosome, T> Send for PyCodec<C, T> {}
unsafe impl<C: Chromosome, T> Sync for PyCodec<C, T> {}

pub(super) fn decode_genotype_to_array<'py, C, G, A>(
    py: Python<'py>,
    genotype: &Genotype<C>,
    use_numpy: bool,
) -> PyResult<Bound<'py, PyAny>>
where
    C: Chromosome<Gene = G>,
    G: Gene<Allele = A>,
    A: Element + IntoPyObject<'py> + Copy,
{
    let lengths = genotype
        .iter()
        .map(|chrom| chrom.len())
        .collect::<Vec<usize>>();

    if genotype.len() == 1 {
        let values = genotype
            .iter()
            .next()
            .map(|chrom| chrom.iter().map(|gene| *gene.allele()).collect::<Vec<A>>())
            .unwrap_or_default();

        let is_square = lengths.iter().all(|&len| len == lengths[0]);

        if is_square && use_numpy {
            return match lengths.len() {
                1 => Ok(PyArray1::from_vec(py, values).into_any()),
                _ => Ok(PyArray::from_iter(py, values)
                    .reshape([lengths.len(), lengths[0]])?
                    .into_any()),
            };
        }

        return Ok(PyList::new(py, values)?.into_any());
    }

    let is_square = lengths.iter().all(|&len| len == lengths[0]);

    if use_numpy && is_square {
        let values = genotype
            .iter()
            .flat_map(|chrom| chrom.iter().map(|gene| *gene.allele()))
            .collect::<Vec<A>>();

        return match lengths.len() {
            1 => Ok(PyArray1::from_vec(py, values).into_any()),
            _ => Ok(PyArray::from_iter(py, values)
                .reshape([lengths.len(), lengths[0]])?
                .into_any()),
        };
    }

    let values = genotype
        .iter()
        .map(|chrom| chrom.iter().map(|gene| *gene.allele()).collect::<Vec<A>>())
        .collect::<Vec<Vec<A>>>();

    let outer = PyList::empty(py);
    for chromo in values.iter() {
        let inner = PyList::empty(py);
        for gene in chromo.iter() {
            inner.append(*gene).unwrap();
        }
        outer.append(inner).unwrap();
    }

    Ok(outer.into_any())
}
