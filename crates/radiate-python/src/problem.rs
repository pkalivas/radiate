use pyo3::types::PyAny;
use pyo3::{prelude::*, types::PyDict};
use radiate::{AnyChromosome, Chromosome, Genotype, Problem, Score};

use crate::gene::PyGeneotype;

#[pyclass(name = "Problem", unsendable)]
pub struct PyProblem {
    pub py_obj: Py<PyAny>,
}

impl PyProblem {
    pub fn new(py_obj: Py<PyAny>) -> Self {
        Self { py_obj }
    }
}

// impl<'a> Problem<AnyChromosome<'a>, PyObject> for PyProblem {
//     fn encode(&self) -> Genotype<AnyChromosome<'a>> {
//         Python::with_gil(|py| {
//             let genotype = self
//                 .py_obj
//                 .as_any()
//                 .call_method0(py, "encode")
//                 .expect("encode failed")
//                 .extract::<PyGeneotype>(py)
//                 .expect("encode returned invalid Genotype");

//             Genotype::from(genotype.take())
//         })
//     }

//     fn decode(&self, genotype: &Genotype<AnyChromosome>) -> PyObject {
//         // Python::with_gil(|py| {
//         //     self.py_obj
//         //         .as_any()
//         //         .call_method1(py, "decode", (genotype.clone(),))
//         //         .expect("decode failed")
//         //         .extract()
//         //         .expect("decode returned wrong type")
//         // })

//         panic!()
//     }

//     fn eval(&self, genotype: &Genotype<AnyChromosome>) -> Score {
//         panic!()
//         // Python::with_gil(|py| {
//         //     self.py_obj
//         //         .as_ref(py)
//         //         .call_method1("eval", (genotype.clone(),))
//         //         .expect("eval failed")
//         //         .extract::<f32>()
//         //         .expect("eval returned non-f32")
//         //         .into()
//         // })
//     }
// }

// // impl<C> Problem<C, PyDict> for PyProblem<C>
// // where
// //     C: Chromosome + Send + Sync,
// // {
// //     fn encode(&self) -> Genotype<C> {
// //         Python::with_gil(|py| {
// //             self.py_obj
// //                 .as_any()
// //                 .call_method0(py, "encode")
// //                 .expect("encode failed")
// //                 .extract()
// //                 .expect("encode returned invalid Genotype")
// //         })
// //     }

// //     fn decode(&self, genotype: &Genotype<C>) -> PyDict {
// //         Python::with_gil(|py| {
// //             self.py_obj
// //                 .as_ref(py)
// //                 .call_method1("decode", (genotype.clone(),))
// //                 .expect("decode failed")
// //                 .extract()
// //                 .expect("decode returned wrong type")
// //         })
// //     }

// //     fn eval(&self, genotype: &Genotype<C>) -> Score {
// //         Python::with_gil(|py| {
// //             self.py_obj
// //                 .as_ref(py)
// //                 .call_method1("eval", (genotype.clone(),))
// //                 .expect("eval failed")
// //                 .extract::<f32>()
// //                 .expect("eval returned non-f32")
// //                 .into()
// //         })
// //     }
// // }
