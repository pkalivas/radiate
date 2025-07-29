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
pub use graph::{PyGraph, PyGraphCodec};
pub use int::PyIntCodec;
pub use permutation::PyPermutationCodec;
pub use tree::{PyTree, PyTreeCodec};

use pyo3::Python;
use radiate::{Chromosome, Codec, Genotype};

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
        Python::with_gil(|py| match &self.decoder {
            Some(decoder) => decoder(py, genotype),
            None => panic!("Decoder function is not set"),
        })
    }
}

unsafe impl<C: Chromosome, T> Send for PyCodec<C, T> {}
unsafe impl<C: Chromosome, T> Sync for PyCodec<C, T> {}
