use super::PyCodec;
use crate::{PyAnyObject, PyGenotype, Wrap};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods};
use radiate::{BitChromosome, Codec, DataType, Genotype};

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyBitCodec {
    pub codec: PyCodec<BitChromosome, PyAnyObject>,
}

#[pymethods]
impl PyBitCodec {
    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        Ok(PyGenotype::from(self.codec.encode()))
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

    pub fn dtype<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Wrap(DataType::Boolean).into_bound_py_any(py)
    }

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, use_numpy=false))]
    pub fn matrix(chromosome_lengths: Option<Vec<usize>>, use_numpy: bool) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let lengths_for_encoder = lengths.clone();

        PyBitCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    lengths_for_encoder
                        .iter()
                        .map(|len| BitChromosome::new(*len))
                        .collect::<Vec<BitChromosome>>()
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
    #[pyo3(signature = (chromosome_length=1, use_numpy=false))]
    pub fn vector(chromosome_length: Option<usize>, use_numpy: bool) -> Self {
        let length = chromosome_length.unwrap_or(1);

        PyBitCodec {
            codec: PyCodec::new()
                .with_encoder(move || Genotype::from(vec![BitChromosome::new(length)]))
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
                }),
        }
    }
}

unsafe impl Send for PyBitCodec {}
unsafe impl Sync for PyBitCodec {}
