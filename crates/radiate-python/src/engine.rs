use pyo3::{Bound, PyAny, PyResult, intern, pyclass, pymethods, types::PyAnyMethods};
use radiate::{EngineExt, Epoch, FloatChromosome, FloatCodex, Generation, GeneticEngine};

#[pyclass(name = "FloatEngine", unsendable)]
pub struct PyFloatEngine {
    pub inner: GeneticEngine<FloatChromosome, f32, Generation<FloatChromosome, f32>>,
}

#[pymethods]
impl PyFloatEngine {
    #[new]
    pub fn new() -> PyResult<Self> {
        let engine = GeneticEngine::builder()
            .codex(FloatCodex::scalar(0.0..100.0))
            .fitness_fn(|x: f32| x)
            .build();
        Ok(Self { inner: engine })
    }

    pub fn run(&mut self, generations: usize) -> PyResult<()> {
        let res = self.inner.run(|output| {
            println!("Generation: {}", output.index());
            output.index() == generations
        });
        // self.inner
        //     .iter()
        //     .take(generations)
        //     .inspect(|ctx| {
        //         println!("Generation: {}", ctx.index());
        //     })
        //     .last();
        Ok(())
    }
}
