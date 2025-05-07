use crate::{AnyValue, PyEngineBuilder, PyFloatCodex, ThreadSafePythonFn};
use pyo3::{PyObject, Python, pyclass, pymethods};
use radiate::{
    Chromosome, EngineExt, Epoch, FloatChromosome, FnCodex, Gene, GeneticEngine, log_ctx,
    steps::SequentialEvaluator,
};

#[pyclass]
pub struct PyFloatEngine {
    pub engine: GeneticEngine<FloatChromosome, Vec<Vec<f32>>>,
}

#[pymethods]
impl PyFloatEngine {
    #[new]
    #[pyo3(signature = (codex, fitness_func, builder))]
    pub fn new(codex: PyFloatCodex, fitness_func: PyObject, builder: PyEngineBuilder) -> Self {
        let codex = FnCodex::new()
            .with_encoder(move || {
                codex
                    .chromosome_lengths
                    .iter()
                    .map(|len| {
                        FloatChromosome::from((
                            *len,
                            codex.value_range.clone(),
                            codex.bound_range.clone(),
                        ))
                    })
                    .collect::<Vec<FloatChromosome>>()
                    .into()
            })
            .with_decoder(|geno| {
                geno.iter()
                    .map(|chromo| {
                        chromo
                            .iter()
                            .map(|gene| *gene.allele())
                            .collect::<Vec<f32>>()
                    })
                    .collect::<Vec<Vec<f32>>>()
            });

        let fitness = ThreadSafePythonFn::new(fitness_func);

        let mut engine = GeneticEngine::builder()
            .codex(codex)
            .minimizing()
            .evaluator(SequentialEvaluator)
            .fitness_fn(move |decoded: Vec<Vec<f32>>| {
                Python::with_gil(|py| {
                    let wrapped_decoded = AnyValue::from(decoded);
                    fitness.call(py, wrapped_decoded)
                })
            })
            .population_size(builder.population_size);

        engine = crate::set_selector(engine, &builder.offspring_selector, true);
        engine = crate::set_selector(engine, &builder.survivor_selector, false);
        engine = crate::get_alters_with_arithmetic_gene(engine, &builder.alters);

        PyFloatEngine {
            engine: engine.build(),
        }
    }

    pub fn run(&mut self, generations: usize) {
        let engine = &mut self.engine;
        engine.run(|epoch| {
            log_ctx!(epoch);
            epoch.index() > generations
        });
    }
}
