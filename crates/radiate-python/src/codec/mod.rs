mod bit;
mod char;
mod float;
mod int;

use std::sync::Arc;

use crate::{
    ObjectValue,
    conversion::{any_value_into_py_object, py_object_to_any_value},
};
pub use bit::PyBitCodec;
pub use char::PyCharCodec;
pub use float::PyFloatCodec;
pub use int::PyIntCodec;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{Chromosome, Codec, Genotype};

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

#[pyclass]
pub struct PyAnyCodec;

#[pymethods]
impl PyAnyCodec {
    #[new]
    pub fn new() -> Self {
        PyAnyCodec
    }

    pub fn test<'py>(&self, py: Python<'py>, value: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        // This is a placeholder for any operation you want to perform with the value.
        // For now, it just returns the value back.

        let any_vals = py_object_to_any_value(value.bind(py), true).unwrap();
        println!("Received value: {:?}", any_vals);

        let result = any_value_into_py_object(any_vals, py).unwrap();

        Ok(result)
    }
}
