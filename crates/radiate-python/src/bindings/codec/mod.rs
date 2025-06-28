mod bit;
mod char;
mod float;
mod graph;
mod int;

use std::sync::Arc;

use crate::{ObjectValue, conversion::Wrap};
pub use bit::PyBitCodec;
pub use char::PyCharCodec;
pub use float::PyFloatCodec;
pub use graph::PyGraphCodec;
pub use int::PyIntCodec;
use pyo3::{Bound, FromPyObject, PyAny, PyResult, Python, types::PyAnyMethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, Codec, FloatChromosome, Genotype, GraphChromosome,
    IntChromosome, Op,
};

#[derive(Clone)]
pub struct PyCodec<C: Chromosome> {
    encoder: Option<Arc<dyn Fn() -> Genotype<C>>>,
    decoder: Option<Arc<dyn Fn(Python<'_>, &Genotype<C>) -> ObjectValue>>,
}

impl<C: Chromosome> PyCodec<C> {
    pub fn new() -> Self {
        PyCodec {
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

impl<C: Chromosome> Codec<C, ObjectValue> for PyCodec<C> {
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

unsafe impl<C: Chromosome> Send for PyCodec<C> {}
unsafe impl<C: Chromosome> Sync for PyCodec<C> {}

impl<'py> FromPyObject<'py> for Wrap<PyCodec<FloatChromosome>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let codec_attr = ob.getattr("codec")?;

        if codec_attr.is_instance_of::<PyFloatCodec>() {
            let codec = codec_attr.extract::<PyFloatCodec>()?.codec;
            return Ok(Wrap(codec));
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a PyFloatCodec, but got a different codec type",
        ))
    }
}

impl<'py> FromPyObject<'py> for Wrap<PyCodec<CharChromosome>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let codec_attr = ob.getattr("codec")?;

        if codec_attr.is_instance_of::<PyCharCodec>() {
            let codec = codec_attr.extract::<PyCharCodec>()?.codec;
            return Ok(Wrap(codec));
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a PyCharCodec, but got a different codec type",
        ))
    }
}

impl<'py> FromPyObject<'py> for Wrap<PyCodec<IntChromosome<i32>>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let codec_attr = ob.getattr("codec")?;

        if codec_attr.is_instance_of::<PyIntCodec>() {
            let codec = codec_attr.extract::<PyIntCodec>()?.codec;
            return Ok(Wrap(codec));
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a PyIntCodec, but got a different codec type",
        ))
    }
}

impl<'py> FromPyObject<'py> for Wrap<PyCodec<BitChromosome>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let codec_attr = ob.getattr("codec")?;

        if codec_attr.is_instance_of::<PyBitCodec>() {
            let codec = codec_attr.extract::<PyBitCodec>()?.codec;
            return Ok(Wrap(codec));
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a PyBitCodec, but got a different codec type",
        ))
    }
}

impl<'py> FromPyObject<'py> for Wrap<PyCodec<GraphChromosome<Op<f32>>>> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let codec_attr = ob.getattr("codec")?;

        if codec_attr.is_instance_of::<PyGraphCodec>() {
            let codec = codec_attr.extract::<PyGraphCodec>()?.codec;
            return Ok(Wrap(codec));
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a PyCharCodec, but got a different codec type",
        ))
    }
}
