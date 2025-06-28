mod bit;
mod char;
mod float;
mod graph;
mod int;

use std::sync::Arc;

use crate::{ObjectValue, PyChromosomeType, PyGeneType, conversion::Wrap};
pub use bit::PyBitCodec;
pub use char::PyCharCodec;
pub use float::PyFloatCodec;
pub use graph::{PyGraph, PyGraphCodec};
pub use int::PyIntCodec;

use pyo3::{Bound, FromPyObject, PyAny, PyResult, Python, pyclass, types::PyAnyMethods};
use radiate::{
    BitChromosome, CharChromosome, Chromosome, Codec, FloatChromosome, Genotype, Graph,
    GraphChromosome, IntChromosome, Op,
};

#[pyclass(unsendable)]
#[derive(Clone, Debug)]
pub struct PyCodecInner {
    pub name: String,
    pub type_name: String,
    pub args: ObjectValue,
    pub gene_types: Vec<PyGeneType>,
    pub chromosome_types: Vec<PyChromosomeType>,
}

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

impl<'py> FromPyObject<'py> for Wrap<PyCodec<FloatChromosome, ObjectValue>> {
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

impl<'py> FromPyObject<'py> for Wrap<PyCodec<CharChromosome, ObjectValue>> {
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

impl<'py> FromPyObject<'py> for Wrap<PyCodec<IntChromosome<i32>, ObjectValue>> {
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

impl<'py> FromPyObject<'py> for Wrap<PyCodec<BitChromosome, ObjectValue>> {
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

impl<'py> FromPyObject<'py> for Wrap<PyCodec<GraphChromosome<Op<f32>>, Graph<Op<f32>>>> {
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
