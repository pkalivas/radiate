// use crate::conversion::Wrap;
// use pyo3::{
//     FromPyObject, IntoPyObject, IntoPyObjectExt, PyAny, Python,
//     types::{PyAnyMethods, PyFloat, PyInt},
// };
// use radiate::FloatGene;

// use super::PyGene;

// impl<'py> IntoPyObject<'py> for Wrap<FloatGene> {
//     type Target = PyAny;
//     type Output = pyo3::Bound<'py, Self::Target>;
//     type Error = pyo3::PyErr;

//     fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
//         PyGene::from(self.0.clone()).into_bound_py_any(py)
//     }
// }

// impl<'py> IntoPyObject<'py> for Wrap<&FloatGene> {
//     type Target = PyAny;
//     type Output = pyo3::Bound<'py, Self::Target>;
//     type Error = pyo3::PyErr;

//     fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
//         PyGene::from(self.0.clone()).into_bound_py_any(py)
//     }
// }

// impl<'py> FromPyObject<'py> for Wrap<FloatGene> {
//     fn extract_bound(ob: &pyo3::Bound<'py, PyAny>) -> pyo3::PyResult<Self> {
//         if ob.is_instance_of::<PyFloat>() || ob.is_instance_of::<PyInt>() {
//             let allele = ob
//                 .extract::<f64>()
//                 .unwrap_or_else(|_| ob.extract::<i64>().unwrap() as f64);
//             return Ok(Wrap(FloatGene::new(
//                 allele as f32,
//                 std::f32::MIN..std::f32::MAX,
//                 std::f32::MIN..std::f32::MAX,
//             )));
//         }

//         let allele = if let Ok(allele) = ob.get_item("allele")?.extract::<f64>() {
//             allele as f32
//         } else if let Ok(allele) = ob.get_item("allele")?.extract::<f32>() {
//             allele as f32
//         } else {
//             return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                 "Expected a float value for FloatGene allele",
//             ));
//         };

//         let min = if let Ok(min) = ob.get_item("min")?.extract::<f64>() {
//             min as f32
//         } else if let Ok(min) = ob.get_item("min")?.extract::<f32>() {
//             min as f32
//         } else {
//             std::f32::MIN
//         };

//         let max = if let Ok(max) = ob.get_item("max")?.extract::<f64>() {
//             max as f32
//         } else if let Ok(max) = ob.get_item("max")?.extract::<f32>() {
//             max as f32
//         } else {
//             std::f32::MAX
//         };

//         let start_bound = if let Ok(start_bound) = ob.get_item("start_bound")?.extract::<f64>() {
//             start_bound as f32
//         } else if let Ok(start_bound) = ob.get_item("start_bound")?.extract::<i64>() {
//             start_bound as f32
//         } else {
//             std::f32::MIN
//         };

//         let end_bound = if let Ok(end_bound) = ob.get_item("end_bound")?.extract::<f64>() {
//             end_bound as f32
//         } else if let Ok(end_bound) = ob.get_item("end_bound")?.extract::<i64>() {
//             end_bound as f32
//         } else {
//             std::f32::MAX
//         };

//         Ok(Wrap(FloatGene::new(
//             allele,
//             min..max,
//             start_bound..end_bound,
//         )))
//     }
// }
