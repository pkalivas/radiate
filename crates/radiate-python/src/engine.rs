use crate::{
    AnyValue, DataType, Field, PyEngineParam, PySelector, ThreadSafePythonFn,
    get_alters_with_arithmetic_gene,
};
use pyo3::{PyObject, PyResult, Python, pyclass, pymethods};
use radiate::{
    Chromosome, EliteSelector, EngineExt, Epoch, FloatChromosome, FloatCodex, GeneticEngine,
    GeneticEngineBuilder, RankSelector, RouletteSelector, TournamentSelector, log_ctx,
    steps::SequenctialEvaluator,
};

pub trait PyEngineTrait {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn into_inner(self: Box<Self>) -> Box<dyn std::any::Any>;
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[pyclass(unsendable)]
pub struct PyEngine {
    pub chromosome: Field,
    pub target: Field,
    pub epoch: Field,
    pub engine: Option<Box<dyn PyEngineTrait>>,
}

#[pymethods]
impl PyEngine {
    pub fn run(&mut self, generations: usize) {
        let maybe_engine = self.engine.take();
        if let Some(engine) = maybe_engine {
            let mut engine = engine
                .into_inner()
                .downcast::<GeneticEngine<FloatChromosome, Vec<Vec<f32>>>>()
                .unwrap();

            engine.run(|epoch| {
                log_ctx!(epoch);
                epoch.index() > generations
            });
        }
    }

    pub fn __next__(&mut self) {
        let maybe_engine = self.engine.take();
        if let Some(engine) = maybe_engine {
            let mut engine = engine
                .into_inner()
                .downcast::<GeneticEngine<FloatChromosome, Vec<Vec<f32>>>>()
                .unwrap();

            // let epoch = engine.next();
            // log_ctx!(epoch);
        }
    }
}

#[pymethods]
impl PyEngine {
    #[staticmethod]
    #[pyo3(signature = (num_genes, num_chromosomes, objective, fitness_fn, range=None, bounds=None,
    survivor_selector=None, offspring_selector=None, alters=None))]
    pub fn try_build_float_engine<'py>(
        num_genes: usize,
        num_chromosomes: usize,
        objective: Option<Vec<String>>,
        fitness_fn: PyObject,
        range: Option<(f32, f32)>,
        bounds: Option<(f32, f32)>,
        survivor_selector: Option<PySelector>,
        offspring_selector: Option<PySelector>,
        alters: Option<Vec<PyEngineParam>>,
    ) -> PyResult<PyEngine> {
        let fitness_fn = ThreadSafePythonFn::new(fitness_fn);

        let range = range.map(|(min, max)| min..max).unwrap_or(0.0..1.0);
        let bounds = bounds.map(|(min, max)| min..max).unwrap_or(range.clone());

        let codex = FloatCodex::matrix(num_chromosomes, num_genes, range).with_bounds(bounds);

        let mut builder = GeneticEngine::builder()
            .codex(codex)
            .evaluator(SequenctialEvaluator)
            .fitness_fn(move |decoded: Vec<Vec<f32>>| {
                Python::with_gil(|py| {
                    let wrapped_decoded = AnyValue::from(decoded);
                    let result = fitness_fn.call(py, wrapped_decoded);

                    result
                })
            });

        if let Some(surv_selector) = survivor_selector {
            builder = set_selector(builder, surv_selector, false);
        }

        if let Some(offs_selector) = offspring_selector {
            builder = set_selector(builder, offs_selector, true);
        }

        if let Some(alters) = alters {
            let alters = get_alters_with_arithmetic_gene(alters);
            builder = builder.alter(alters);
        }

        if let Some(objectives) = objective {
            builder = builder.minimizing();
        }

        let chromosome_field = Field::new(
            std::any::type_name::<FloatChromosome>().to_string(),
            DataType::Struct(vec![
                Field::new("allele".to_string(), DataType::Float32),
                Field::new("fitness".to_string(), DataType::Float32),
            ]),
        );

        let target_field = Field::new(
            std::any::type_name::<Vec<Vec<f32>>>().to_string(),
            DataType::List(Box::new(Field::new(
                "target".to_string(),
                DataType::Float32,
            ))),
        );
        let epoch_field = Field::new(std::any::type_name::<usize>().to_string(), DataType::Int32);

        Ok(PyEngine {
            chromosome: chromosome_field,
            target: target_field,
            epoch: epoch_field,
            engine: Some(Box::new(builder.build())),
        })
    }
}

fn set_selector<C, T>(
    builder: GeneticEngineBuilder<C, T>,
    selector: PySelector,
    is_offspring: bool,
) -> GeneticEngineBuilder<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync,
{
    if selector.name() == "tournament" {
        let args = selector.get_args();
        let tournament_size = args
            .get("k")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        return match is_offspring {
            true => builder.offspring_selector(TournamentSelector::new(tournament_size)),
            false => builder.survivor_selector(TournamentSelector::new(tournament_size)),
        };
    } else if selector.name() == "roulette" {
        return match is_offspring {
            true => builder.offspring_selector(RouletteSelector::new()),
            false => builder.survivor_selector(RouletteSelector::new()),
        };
    } else if selector.name() == "rank" {
        return match is_offspring {
            true => builder.offspring_selector(RankSelector::new()),
            false => builder.survivor_selector(RankSelector::new()),
        };
    } else if selector.name() == "elitism" {
        return match is_offspring {
            true => builder.offspring_selector(EliteSelector::new()),
            false => builder.survivor_selector(RouletteSelector::new()),
        };
    }

    builder
}

macro_rules! impl_py_eng {
    ($engine:ty) => {
        impl PyEngineTrait for $engine {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn into_inner(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }
        }
    };
}

impl_py_eng!(GeneticEngine<FloatChromosome, Vec<Vec<f32>>>);
