use crate::{AnyValue, conversion::Wrap};
use pyo3::{pyclass, pymethods};
use radiate::{
    BitChromosome, BitGene, CharChromosome, CharGene, FloatChromosome, Gene, IntChromosome,
};

// #[pyclass(name = "AnyChromosome")]
// #[repr(transparent)]
// #[derive(Clone, Debug)]
// pub struct PyAnyChromosome {
//     pub inner: AnyChromosome<'static>,
// }

// #[pymethods]
// impl PyAnyChromosome {
//     #[new]
//     #[pyo3(signature = (allele))]
//     pub fn new(allele: Vec<Option<Wrap<AnyValue<'_>>>>) -> Self {
//         let chromosome = AnyChromosome::new(
//             allele
//                 .iter()
//                 .map(|val| {
//                     if let Some(val) = val {
//                         val.0.clone().into_static()
//                     } else {
//                         AnyValue::Null
//                     }
//                 })
//                 .collect(),
//         );

//         PyAnyChromosome { inner: chromosome }
//     }

//     pub fn __str__(&self) -> String {
//         self.inner
//             .as_ref()
//             .iter()
//             .map(|gene| gene.allele().type_name().to_string())
//             .collect::<Vec<String>>()
//             .join(", ")
//     }
// }
