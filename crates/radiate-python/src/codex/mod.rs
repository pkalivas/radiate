mod bit;
mod char;
mod float;
mod int;

use std::sync::Arc;

use crate::conversion::ObjectValue;
pub use bit::PyBitCodex;
pub use char::PyCharCodex;
pub use float::PyFloatCodex;
pub use int::PyIntCodex;
use pyo3::Python;
use radiate::{Chromosome, Codex, Genotype};

#[derive(Clone)]
pub struct PyCodex<C: Chromosome> {
    encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    decoder: Option<Arc<dyn Fn(Python<'_>, &Genotype<C>) -> ObjectValue>>,
}

impl<C: Chromosome> PyCodex<C> {
    pub fn new() -> Self {
        PyCodex {
            encoder: None,
            decoder: None,
        }
    }

    pub fn decode_with_py(&self, py: Python<'_>, genotype: &Genotype<C>) -> ObjectValue {
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
        F: Fn(Python<'_>, &Genotype<C>) -> ObjectValue + 'static,
    {
        self.decoder = Some(Arc::new(decoder));
        self
    }
}

impl<C: Chromosome> Codex<C, ObjectValue> for PyCodex<C> {
    fn encode(&self) -> Genotype<C> {
        match &self.encoder {
            Some(encoder) => encoder(),
            None => panic!("Encoder function is not set"),
        }
    }

    fn decode(&self, genotype: &Genotype<C>) -> ObjectValue {
        Python::with_gil(|py| match &self.decoder {
            Some(decoder) => decoder(py, genotype),
            None => panic!("Decoder function is not set"),
        })
    }
}
