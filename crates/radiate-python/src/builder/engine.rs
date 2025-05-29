use crate::{PyEngineBuilder, PyEvaluator, PyProblem, codec::PyCodec, conversion::ObjectValue};
use pyo3::PyObject;
use radiate::{
    Chromosome, GeneticEngine, GeneticEngineBuilder, MultiObjectiveGeneration, Optimize,
    steps::SequentialEvaluator,
};

#[allow(dead_code)]
pub(crate) fn build_multi_objective_engine<C>(
    codec: PyCodec<C>,
    fitness_func: PyObject,
    builder: &PyEngineBuilder,
) -> GeneticEngineBuilder<C, ObjectValue, MultiObjectiveGeneration<C>>
where
    C: Chromosome + PartialEq + Clone,
{
    let mut engine = GeneticEngine::builder()
        .problem(PyProblem::new(fitness_func, codec))
        .population_size(builder.population_size);

    engine = set_evaluator(engine, &builder.num_threads);
    engine = crate::set_selector(engine, &builder.offspring_selector, true);
    engine = crate::set_selector(engine, &builder.survivor_selector, false);

    engine.multi_objective(
        builder
            .objectives
            .iter()
            .map(|ob| match ob.to_lowercase().trim() {
                "min" => Optimize::Minimize,
                "max" => Optimize::Maximize,
                _ => panic!("Invalid objective {}", ob),
            })
            .collect::<Vec<Optimize>>(),
    )
}

pub fn set_evaluator<C, T>(
    builder: GeneticEngineBuilder<C, T>,
    num_threads: &usize,
) -> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send + Sync,
{
    match num_threads {
        1 => builder.evaluator(SequentialEvaluator),
        _ => builder.num_threads(*num_threads).evaluator(PyEvaluator),
    }
}
