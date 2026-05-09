use super::PyCodec;
use crate::{PyAnyObject, PyGenotype};
use pyo3::{Bound, IntoPyObjectExt, PyAny, PyResult, pyclass, pymethods};
use radiate::{BitChromosome, Codec, Genotype};

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

    #[staticmethod]
    #[pyo3(signature = (chromosome_lengths=None, use_numpy=false))]
    pub fn matrix(chromosome_lengths: Option<Vec<usize>>, use_numpy: bool) -> Self {
        let lengths = chromosome_lengths.unwrap_or(vec![1]);
        let lengths_for_encoder = lengths.clone();
        let lengths_for_write = lengths.clone();

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
                })
                .with_write(move |w| bit_codec_write(w, &lengths_for_write, use_numpy)),
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
                })
                .with_write(move |w| bit_codec_write(w, &[length], use_numpy)),
        }
    }
}

fn bit_codec_write(
    w: &mut dyn std::io::Write,
    lengths: &[usize],
    use_numpy: bool,
) -> std::io::Result<()> {
    writeln!(w, "type: BitCodec")?;
    writeln!(w, "shape: {:?}", lengths)?;
    writeln!(w, "use_numpy: {}", use_numpy)
}

unsafe impl Send for PyBitCodec {}
unsafe impl Sync for PyBitCodec {}
