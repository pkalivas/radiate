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
pub use graph::PyGraphCodec;
pub use int::PyIntCodec;
pub use permutation::PyPermutationCodec;
pub use tree::PyTreeCodec;

use numpy::{Element, PyArray1};
use pyo3::{Bound, IntoPyObject, PyAny, PyResult, types::PyList};
use pyo3::{IntoPyObjectExt, Python};
use radiate::{Chromosome, Codec, Gene, Genotype};

#[derive(Clone)]
pub struct PyCodec<C: Chromosome, T> {
    encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    decoder: Option<Arc<dyn for<'py> Fn(Python<'py>, &Genotype<C>) -> T + Send + Sync>>,
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
    A: Element + IntoPyObject<'py> + IntoPyObjectExt<'py> + Copy + Default,
{
    // Scalar branceh: if there's only one gene with one allele, return it directly as a scalar
    if genotype.len() == 1 && genotype[0].len() == 1 {
        let value = genotype[0].get(0).allele();
        return Ok(value.into_py_any(py)?.into_bound(py));
    }

    // Single chromosome branch
    if genotype.len() == 1 {
        let chrom = &genotype[0];
        let mut alleles = Vec::with_capacity(chrom.len());

        for gene in chrom.as_slice() {
            alleles.push(*gene.allele());
        }

        if use_numpy {
            return Ok(PyArray1::from_vec(py, alleles).into_any());
        }

        return Ok(PyList::new(py, alleles)?.into_any());
    }

    // multi-chromosome branches
    let mut outer = Vec::with_capacity(genotype.len());
    for chrom in genotype.iter() {
        let mut alleles = Vec::with_capacity(chrom.len());
        for gene in chrom.as_slice() {
            alleles.push(*gene.allele());
        }

        outer.push(alleles);
    }

    if use_numpy {
        return Ok(PyList::new(
            py,
            outer
                .into_iter()
                .map(|chrom| PyArray1::from_vec(py, chrom).into_any()),
        )?
        .into_any());
    }

    Ok(PyList::new(
        py,
        outer
            .into_iter()
            .map(|chrom| PyList::new(py, chrom).unwrap().into_any()),
    )?
    .into_any())
}
