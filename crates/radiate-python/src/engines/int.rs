use crate::{
    PyEngineBuilder, PyEngineParam, PyGeneration, PyIntCodex, ThreadSafePythonFn,
    conversion::ObjectValue,
};
use pyo3::{
    PyObject, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{Epoch, Generation, GeneticEngine, IntChromosome, steps::SequentialEvaluator};

#[pyclass]
pub struct PyIntEngine {
    pub engine: Option<GeneticEngine<IntChromosome<i32>, ObjectValue>>,
}

#[pymethods]
impl PyIntEngine {
    #[new]
    #[pyo3(signature = (codex, fitness_func, builder))]
    pub fn new(codex: PyIntCodex, fitness_func: PyObject, builder: PyEngineBuilder) -> Self {
        let fitness = ThreadSafePythonFn::new(fitness_func);

        let mut engine = GeneticEngine::builder()
            .codex(codex.codex)
            .num_threads(builder.num_threads)
            .evaluator(SequentialEvaluator)
            .fitness_fn(move |decoded: ObjectValue| {
                Python::with_gil(|py| fitness.call(py, decoded))
            })
            .population_size(builder.population_size);

        engine = crate::set_selector(engine, &builder.offspring_selector, true);
        engine = crate::set_selector(engine, &builder.survivor_selector, false);
        engine = crate::get_alters_with_int_gene(engine, &builder.alters);
        engine = crate::set_single_objective(engine, &builder.objectives);

        PyIntEngine {
            engine: Some(engine.build()),
        }
    }

    pub fn run(&mut self, limits: Vec<PyEngineParam>, log: bool) -> PyResult<PyGeneration> {
        crate::run_single_objective_engine(&mut self.engine, limits, log)
    }
}

impl Into<PyGeneration> for Generation<IntChromosome<i32>, ObjectValue> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                index: self.index(),
                score: score.unbind(),
                value: self.value().clone().inner,
                metrics: self.metrics().clone().into(),
            }
        })
    }
}

// https://pyo3.rs/v0.23.5/parallelism.html

// pub struct PyEvaluator;

// impl<C, T> Evaluator<C, T> for PyEvaluator
// where
//     C: Chromosome + 'static,
//     T: 'static,
// {
//     fn eval(
//         &self,
//         ecosystem: &mut radiate::Ecosystem<C>,
//         thread_pool: Arc<radiate::thread_pool::ThreadPool>,
//         problem: Arc<dyn radiate::Problem<C, T>>,
//     ) -> usize {
//         let mut jobs = Vec::new();
//         let len = ecosystem.population.len();
//         for idx in 0..len {
//             if ecosystem.population[idx].score().is_none() {
//                 let geno = ecosystem.population[idx].take_genotype();
//                 jobs.push((idx, geno));
//             }
//         }

//         let work_results = jobs
//             .into_iter()
//             .map(|(idx, geno)| {
//                 let problem = Arc::clone(&problem);
//                 thread_pool.submit_with_result(move || {
//                     let score = problem.eval(&geno);
//                     (idx, score, geno)
//                 })
//             })
//             .collect::<Vec<_>>();

//         let count = work_results.len();
//         for work_result in work_results {
//             let (idx, score, genotype) = work_result.result();
//             ecosystem.population[idx].set_score(Some(score));
//             ecosystem.population[idx].set_genotype(genotype);
//         }

//         count
//     }
// }

// // These traits let us use int_par_iter and map
// use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

// #[pyclass]
// struct UserID {
//     id: i64,
// }

// let allowed_ids: Vec<bool> = Python::with_gil(|outer_py| {
//     let instances: Vec<Py<UserID>> = (0..10).map(|x| Py::new(outer_py, UserID { id: x }).unwrap()).collect();
//     outer_py.allow_threads(|| {
//         instances.par_iter().map(|instance| {
//             Python::with_gil(|inner_py| {
//                 instance.borrow(inner_py).id > 5
//             })
//         }).collect()
//     })
// });
// assert!(allowed_ids.into_iter().filter(|b| *b).count() == 4);
