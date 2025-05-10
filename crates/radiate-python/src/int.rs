use crate::{
    AnyValue, DataType, Field, Limit, PyEngineBuilder, PyEngineParam, PyGeneration,
    ThreadSafePythonFn, conversion::any_value_into_py_object,
};
use pyo3::{
    PyObject, PyResult, Python, pyclass, pymethods,
    types::{PyList, PyListMethods},
};
use radiate::{
    Chromosome, EngineExt, Epoch, FnCodex, Gene, Generation, GeneticEngine, IntChromosome, log_ctx,
    steps::SequentialEvaluator,
};
use std::ops::Range;

#[pyclass]
#[derive(Clone)]
pub struct PyIntCodex {
    pub chromosome_lengths: Vec<usize>,
    pub value_range: Range<i32>,
    pub bound_range: Range<i32>,
}

#[pymethods]
impl PyIntCodex {
    #[new]
    #[pyo3(signature = (chromosome_lengths=None, value_range=None, bound_range=None))]
    pub fn new(
        chromosome_lengths: Option<Vec<usize>>,
        value_range: Option<(i32, i32)>,
        bound_range: Option<(i32, i32)>,
    ) -> Self {
        let val_range = value_range.unwrap_or((0, 1));
        let bound_range = bound_range.unwrap_or(val_range);
        PyIntCodex {
            chromosome_lengths: chromosome_lengths.unwrap_or(vec![1]),
            value_range: val_range.0..val_range.1,
            bound_range: bound_range.0..bound_range.1,
        }
    }
}

#[pyclass]
pub struct PyIntEngine {
    pub engine: GeneticEngine<IntChromosome<i32>, AnyValue<'static>>,
}

#[pymethods]
impl PyIntEngine {
    #[new]
    #[pyo3(signature = (codex, fitness_func, builder))]
    pub fn new(codex: PyIntCodex, fitness_func: PyObject, builder: PyEngineBuilder) -> Self {
        let codex = FnCodex::new()
            .with_encoder(move || {
                codex
                    .chromosome_lengths
                    .iter()
                    .map(|len| {
                        IntChromosome::from((
                            *len,
                            codex.value_range.clone(),
                            codex.bound_range.clone(),
                        ))
                    })
                    .collect::<Vec<IntChromosome<i32>>>()
                    .into()
            })
            .with_decoder(|geno| {
                let mut list = Vec::new();
                for chromo in geno.iter() {
                    let mut genes = Vec::new();
                    for gene in chromo.iter() {
                        genes.push(AnyValue::from(*gene.allele()));
                    }
                    list.push(AnyValue::VecOwned(Box::new((
                        genes,
                        Field::new(
                            std::any::type_name::<Vec<i32>>().to_string(),
                            DataType::List(Box::new(Field::new(
                                "item".to_string(),
                                DataType::Null,
                            ))),
                        ),
                    ))));
                }

                AnyValue::VecOwned(Box::new((
                    list,
                    Field::new(
                        std::any::type_name::<Vec<Vec<i32>>>().to_string(),
                        DataType::List(Box::new(Field::new("item".to_string(), DataType::Null))),
                    ),
                )))

                // geno.iter()
                //     .map(|chromo| {
                //         chromo
                //             .iter()
                //             .map(|gene| *gene.allele())
                //             .collect::<Vec<i32>>()
                //     })
                //     .collect::<Vec<Vec<i32>>>()
            });

        let fitness = ThreadSafePythonFn::new(fitness_func);

        let mut engine = GeneticEngine::builder()
            .codex(codex)
            .minimizing()
            .evaluator(SequentialEvaluator)
            .fitness_fn(move |decoded: AnyValue<'_>| {
                Python::with_gil(|py| fitness.call(py, decoded))
            })
            .population_size(builder.population_size);

        engine = crate::set_selector(engine, &builder.offspring_selector, true);
        engine = crate::set_selector(engine, &builder.survivor_selector, false);
        engine = crate::get_alters_with_int_gene(engine, &builder.alters);

        PyIntEngine {
            engine: engine.build(),
        }
    }

    pub fn run(&mut self, limits: Vec<PyEngineParam>, log: bool) -> PyResult<PyGeneration> {
        let lims = limits
            .into_iter()
            .map(|lim| Limit::from(lim))
            .collect::<Vec<_>>();
        let engine = &mut self.engine;
        let result = engine.run(|epoch| {
            if log {
                log_ctx!(epoch);
            }

            for limit in lims.iter() {
                match limit {
                    Limit::Generations(lim) => {
                        if epoch.index() >= *lim {
                            return true;
                        }
                    }
                    Limit::Score(lim) => {
                        if epoch.score().as_f32() <= *lim {
                            return true;
                        }
                    }
                    Limit::Seconds(val) => {
                        if epoch.seconds() >= *val {
                            return true;
                        }
                    }
                }
            }

            false
        });

        if log {
            println!("{:?}", result);
        }

        Ok(result.into())
    }
}

impl Into<PyGeneration> for Generation<IntChromosome<i32>, AnyValue<'static>> {
    fn into(self) -> PyGeneration {
        Python::with_gil(|py| {
            let score = PyList::empty(py);

            for val in self.score().values.iter() {
                score.append(*val).unwrap();
            }

            PyGeneration {
                score: score.unbind(),
                value: any_value_into_py_object(self.value().clone(), py)
                    .unwrap()
                    .unbind(),
                metrics: self.metrics().clone().into(),
            }
        })
    }
}
