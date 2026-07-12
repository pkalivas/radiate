mod bit;
mod builder;
mod char;
mod float;
mod graph;
mod int;
mod permutation;
mod tree;

use std::sync::Arc;

pub use bit::PyBitCodec;
pub use builder::{NumericCodecBuilder, TypedNumericCodec};
pub use char::PyCharCodec;
pub use float::PyFloatCodec;
pub use graph::{PyGraphCodec, PyGraphCodecInner};
pub use int::PyIntCodec;
pub use permutation::PyPermutationCodec;
pub use tree::{PyTreeCodec, PyTreeCodecInner};

use numpy::{Element, PyArray1};
use pyo3::{Bound, IntoPyObject, PyAny, PyResult, types::PyList};
use pyo3::{IntoPyObjectExt, Python};
use radiate::{Chromosome, Codec, Gene, Genotype};

type DecoderFn<C, T> = Arc<dyn for<'py> Fn(Python<'py>, &Genotype<C>) -> T + Send + Sync>;

#[derive(Clone)]
pub struct PyCodec<C: Chromosome, T> {
    encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    decoder: Option<DecoderFn<C, T>>,
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
        F: for<'py> Fn(Python<'py>, &Genotype<C>) -> T + 'static + Send + Sync,
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

impl<C: Chromosome, T> Default for PyCodec<C, T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<C: Chromosome, T> Send for PyCodec<C, T> {}
unsafe impl<C: Chromosome, T> Sync for PyCodec<C, T> {}

#[inline]
pub(super) fn decode_genotype_to_array<'py, C, G, A>(
    py: Python<'py>,
    genotype: &Genotype<C>,
    use_numpy: bool,
) -> PyResult<Bound<'py, PyAny>>
where
    C: Chromosome<Gene = G>,
    G: Gene<Allele = A>,
    A: Element + IntoPyObject<'py> + Copy + Default,
{
    if genotype.len() == 1 && genotype[0].len() == 1 {
        return match genotype[0].get(0) {
            Some(gene) => Ok(gene.allele().into_bound_py_any(py)?),
            None => Err(pyo3::exceptions::PyValueError::new_err(
                "Genotype has no genes",
            )),
        };
    }

    if genotype.len() == 1 {
        let chrom = &genotype[0];
        let allele_iter = chrom.as_slice().iter().map(|gene| *gene.allele());

        if use_numpy {
            return Ok(PyArray1::from_iter(py, allele_iter).into_any());
        }

        return Ok(PyList::new(py, allele_iter)?.into_any());
    }

    if use_numpy {
        let outer_iter = genotype.iter().map(|chrom| {
            let allele_iter = chrom.as_slice().iter().map(|gene| *gene.allele());
            PyArray1::from_iter(py, allele_iter)
        });
        return Ok(PyList::new(py, outer_iter)?.into_any());
    }

    let outer_iter = genotype.iter().map(|chrom| {
        let allele_iter = chrom.as_slice().iter().map(|gene| *gene.allele());
        PyList::new(py, allele_iter).expect("Failed to create inner PyList")
    });

    Ok(PyList::new(py, outer_iter)?.into_any())
}
