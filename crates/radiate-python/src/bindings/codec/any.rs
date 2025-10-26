use crate::{AnyChromosome, AnyGene, PyAnyObject, PyCodec, PyGene, PyGenotype, prelude::Wrap};
use pyo3::{IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods, types::PyList};
use radiate::{Chromosome, Codec, Gene, Genotype};

#[pyclass]
#[derive(Clone)]
pub struct PyAnyCodec {
    pub codec: PyCodec<AnyChromosome<'static>, PyAnyObject>,
}

#[pymethods]
impl PyAnyCodec {
    #[new]
    pub fn new(genes: Vec<PyGene>, creator: Py<PyAny>) -> PyResult<Self> {
        let call_creator = move |py: Python<'_>, allele: &AnyGene| -> PyResult<PyAnyObject> {
            let obj = creator.call1(
                py,
                (Wrap(allele.allele()).into_py_any(py)?, allele.metadata()),
            )?;

            Ok(PyAnyObject {
                inner: obj.into_any(),
            })
        };

        Ok(PyAnyCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(
                        genes
                            .iter()
                            .map(|v| AnyGene::from(v.clone()))
                            .collect::<AnyChromosome<'static>>(),
                    )
                })
                .with_decoder(move |py, genotype| {
                    if genotype.len() == 1 && genotype[0].len() == 1 {
                        return call_creator(py, &genotype[0].get(0)).unwrap();
                    }

                    if genotype.len() == 1 {
                        return PyAnyObject {
                            inner: PyList::new(
                                py,
                                genotype
                                    .iter()
                                    .flat_map(|chromo| {
                                        chromo
                                            .iter()
                                            .map(|gene| call_creator(py, gene).unwrap().inner)
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .unwrap()
                            .unbind()
                            .into_any(),
                        };
                    }

                    return PyAnyObject {
                        inner: PyList::new(
                            py,
                            genotype
                                .iter()
                                .map(|chromo| {
                                    chromo
                                        .iter()
                                        .map(|gene| call_creator(py, gene).unwrap().inner)
                                        .collect::<Vec<_>>()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .unwrap()
                        .unbind()
                        .into_any(),
                    };
                }),
        })
    }

    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        Ok(PyGenotype::from(self.codec.encode()))
    }

    pub fn decode_py<'py>(&self, py: Python<'py>, genotype: PyGenotype) -> PyResult<Py<PyAny>> {
        Ok(self.codec.decode_with_py(py, &genotype.into()).inner)
    }
}
