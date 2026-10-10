use crate::bindings::codec::PyCodec;
use crate::{PyAnyObject, PyGenotype};
use numpy::PyArray1;
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, Python, pyclass, pymethods, types::PyList};
use radiate::{Codec, PackedBitChromosome};

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PyPackedBitCodec {
    pub codec: PyCodec<PackedBitChromosome, PyAnyObject>,
}

#[pymethods]
impl PyPackedBitCodec {
    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        Ok(PyGenotype::from(self.codec.encode()))
    }

    pub fn decode_py<'py>(&self, py: Python<'py>, geno: PyGenotype) -> PyResult<Bound<'py, PyAny>> {
        self.codec
            .decode_with_py(py, &geno.clone().into())
            .into_bound_py_any(py)
    }

    /// Decodes to the bits, or with `words=true` to the raw 64-bit words (bit `i` is bit
    /// `i % 64` of word `i / 64`, tail cleared). `use_numpy` picks a numpy array
    /// (`np.bool_` or `np.uint64`) over a Python list (`list[bool]` or `list[int]`).
    #[staticmethod]
    #[pyo3(signature = (num_bits, use_numpy=false, words=false))]
    pub fn vector(num_bits: usize, use_numpy: bool, words: bool) -> Self {
        let codec = PyCodec::new().with_encoder(move || PackedBitChromosome::new(num_bits).into());

        let codec = match (words, use_numpy) {
            (true, true) => codec.with_decoder(move |py, geno| PyAnyObject {
                inner: PyArray1::from_vec(py, geno[0].to_words())
                    .unbind()
                    .into_any(),
            }),
            (true, false) => codec.with_decoder(move |py, geno| PyAnyObject {
                inner: PyList::new(py, &geno[0].to_words())
                    .unwrap()
                    .unbind()
                    .into_any(),
            }),
            (false, true) => codec.with_decoder(move |py, geno| PyAnyObject {
                inner: PyArray1::from_vec(py, geno[0].to_bools())
                    .unbind()
                    .into_any(),
            }),
            (false, false) => codec.with_decoder(move |py, geno| PyAnyObject {
                inner: PyList::new(py, geno[0].to_bools())
                    .unwrap()
                    .unbind()
                    .into_any(),
            }),
        };

        Self { codec }
    }
}
