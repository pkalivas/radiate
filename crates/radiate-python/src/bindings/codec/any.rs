use std::sync::Arc;

use crate::{
    AnyChromosome, AnyGene, AnyValue, PyAnyObject, PyCodec, PyGenotype,
    any::py_object_to_any_value, prelude::Wrap,
};
use pyo3::{
    IntoPyObjectExt, Py, PyAny, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{Chromosome, Codec, Gene, Genotype};
#[pyclass]
#[derive(Clone)]
pub struct PyAnyCodec {
    pub codec: PyCodec<AnyChromosome<'static>, PyAnyObject>,
}

#[pymethods]
impl PyAnyCodec {
    #[new]
    pub fn new(encoder: Py<PyAny>, creator: Py<PyAny>, new_instance: Py<PyAny>) -> PyResult<Self> {
        let allele_factory = Arc::new(move || {
            Python::with_gil(|py| {
                let dict = new_instance.call0(py).expect("new_instance() failed");
                py_object_to_any_value(&dict.into_bound_py_any(py).unwrap(), true)
                    .expect("convert new_instance result")
                    .into_static()
            })
        });
        let codec = PyCodec::new()
            .with_encoder(move || {
                Python::with_gil(|py| {
                    let bound = encoder.as_ref().into_bound_py_any(py).unwrap();
                    let any_val = py_object_to_any_value(&bound, true)
                        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
                        .unwrap();

                    match any_val {
                        AnyValue::Vec(vec) => {
                            let chromos = vec
                                .into_iter()
                                .map(|v| {
                                    let v_static = v.into_static();
                                    let allele_factory = allele_factory.clone();
                                    AnyGene::new(v_static).with_factory(move || allele_factory())
                                })
                                .collect::<AnyChromosome<'static>>();

                            Genotype::from(chromos)
                        }
                        _ => Genotype::from(AnyChromosome::from(vec![AnyGene::new(
                            any_val.into_static(),
                        )])),
                    }
                })
            })
            .with_decoder(move |py, genotype| {
                // helper to call creator(value) -> PyAnyObject
                let call_creator = |py: Python<'_>, allele: &AnyGene| -> PyResult<PyAnyObject> {
                    let obj = creator.call1(py, (Wrap(allele.allele()).into_py_any(py)?,))?;
                    Ok(PyAnyObject {
                        inner: obj.into_any(),
                    })
                };

                // 1 gene
                if genotype.len() == 1 && genotype[0].len() == 1 {
                    return call_creator(py, &genotype[0].get(0)).unwrap();
                }

                // 1 chromosome
                if genotype.len() == 1 {
                    let py_list = PyList::empty(py);
                    for gene in genotype[0].iter() {
                        py_list
                            .append(call_creator(py, &gene).unwrap().inner.as_ref())
                            .unwrap();
                    }
                    return PyAnyObject {
                        inner: py_list.unbind().into_any(),
                    };
                }

                // matrix
                let outer = PyList::empty(py);
                for chromo in genotype.iter() {
                    let inner = PyList::empty(py);
                    for gene in chromo.iter() {
                        inner
                            .append(call_creator(py, &gene).unwrap().inner.as_ref())
                            .unwrap();
                    }
                    outer.append(inner).unwrap();
                }

                PyAnyObject {
                    inner: outer.unbind().into_any(),
                }
            });

        Ok(PyAnyCodec { codec })
    }

    pub fn encode_py(&self) -> PyResult<PyGenotype> {
        Ok(PyGenotype::from(self.codec.encode()))
    }

    pub fn decode_py<'py>(&self, py: Python<'py>, genotype: PyGenotype) -> PyResult<Py<PyAny>> {
        Ok(self.codec.decode_with_py(py, &genotype.into()).inner)
    }
}
