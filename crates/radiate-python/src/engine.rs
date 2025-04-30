use pyo3::{pyclass, pymethods};

#[pyclass(name = "FloatScalarEngine", unsendable)]
pub struct PyFloatScalarEngine {
    // pub inner: GeneticEngine<FloatChromosome, f32, Generation<FloatChromosome, f32>>,
}

#[pymethods]
impl PyFloatScalarEngine {
    #[new]
    pub fn new() -> Self {
        Self {
            // inner: GeneticEngine::new(),
        }
    }

    pub fn name(&self) -> String {
        "HI".to_string()
    }
}
