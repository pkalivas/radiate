use crate::{
    AnyGene, AnyValue, ObjectValue, PyCodec, PyGenotype, any::AnyChromosome,
    object::py_object_to_any_value, prelude::Wrap,
};
use pyo3::{
    IntoPyObjectExt, Py, PyAny, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{Chromosome, Codec, Gene, Genotype};

#[pyclass]
pub struct PyAnyCodec {
    pub codec: PyCodec<AnyChromosome<'static>, ObjectValue>,
}

#[pymethods]
impl PyAnyCodec {
    #[new]
    pub fn new(encoder: Py<PyAny>) -> Self {
        PyAnyCodec {
            codec: PyCodec::new()
                .with_encoder(move || {
                    Python::with_gil(|py| {
                        let enc = encoder.as_ref();
                        let result = enc
                            .call0(py)
                            .expect("Failed to call encoder")
                            .into_bound_py_any(py)
                            .expect("Failed to convert to PyAny");
                        let any_val = py_object_to_any_value(&result, true)
                            .expect("Failed to convert to AnyValue");
                        match any_val {
                            AnyValue::Vec(vec) => {
                                let chromosomes = vec
                                    .into_iter()
                                    .map(|v| AnyGene::new(v.into_static()))
                                    .collect::<AnyChromosome<'static>>();
                                Genotype::from(chromosomes)
                            }
                            _ => Genotype::from(AnyChromosome::from(vec![AnyGene::new(
                                any_val.into_static(),
                            )])),
                        }
                    })
                })
                .with_decoder(move |py, genotype| {
                    if genotype.len() == 1 && genotype[0].len() == 1 {
                        return ObjectValue {
                            inner: Wrap(genotype[0].get(0).allele())
                                .into_py_any(py)
                                .unwrap()
                                .into_any(),
                        };
                    }

                    if genotype.len() == 1 {
                        let chromo = &genotype[0];
                        let py_list = PyList::empty(py);
                        for gene in chromo.iter() {
                            py_list.append(Wrap(gene.allele())).unwrap();
                        }
                        return ObjectValue {
                            inner: py_list.unbind().into_any(),
                        };
                    }

                    let py_list = PyList::empty(py);
                    for chromo in genotype.iter() {
                        let inner = PyList::empty(py);
                        for gene in chromo.iter() {
                            inner.append(Wrap(gene.allele())).unwrap();
                        }
                        py_list.append(inner).unwrap();
                    }
                    ObjectValue {
                        inner: py_list.unbind().into_any(),
                    }
                }),
        }
    }

    pub fn encode_py(&self) -> PyGenotype {
        PyGenotype::from(self.codec.encode())
    }

    pub fn decode_py<'py>(&self, py: Python<'py>, genotype: PyGenotype) -> Py<PyAny> {
        self.codec.decode_with_py(py, &genotype.into()).inner
    }
}
