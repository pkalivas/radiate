use crate::bindings::codec::PyTreeCodec;
use crate::bindings::{EngineBuilderHandle, EngineHandle};
use crate::events::PyEventHandler;
use crate::{
    FreeThreadPyEvaluator, InputConverter, PyCodec, PyEngine, PyEngineInput, PyEngineInputType,
    PyPermutationCodec, prelude::*,
};
use crate::{PyGeneType, PySubscriber};
use core::panic;
use pyo3::exceptions::PyTypeError;
use pyo3::{Py, PyAny, pyclass, pymethods, types::PyAnyMethods};
use radiate::prelude::*;
use std::collections::HashMap;

macro_rules! apply_to_builder {
    ($builder:expr, $method:ident($($args:expr),*)) => {
        match $builder {
            EngineBuilderHandle::Int(b) => Ok(EngineBuilderHandle::Int(b.$method($($args),*))),
            EngineBuilderHandle::Float(b) => Ok(EngineBuilderHandle::Float(b.$method($($args),*))),
            EngineBuilderHandle::Char(b) => Ok(EngineBuilderHandle::Char(b.$method($($args),*))),
            EngineBuilderHandle::Bit(b) => Ok(EngineBuilderHandle::Bit(b.$method($($args),*))),
            EngineBuilderHandle::Permutation(b) => Ok(EngineBuilderHandle::Permutation(b.$method($($args),*))),
            EngineBuilderHandle::Graph(b) => Ok(EngineBuilderHandle::Graph(b.$method($($args),*))),
            EngineBuilderHandle::Tree(b) => Ok(EngineBuilderHandle::Tree(b.$method($($args),*))),
            EngineBuilderHandle::Empty => Err(PyTypeError::new_err(
                "EngineBuilder must have a problem and codec before processing other inputs",
            )),
        }
    };
}

#[pyclass]
pub struct PyEngineBuilder {
    pub gene_type: PyGeneType,
    pub codec: Py<PyAny>,
    pub problem: Py<PyAny>,
    pub subscribers: Vec<PySubscriber>,
    pub inputs: Vec<PyEngineInput>,
}

#[pymethods]
impl PyEngineBuilder {
    #[new]
    pub fn new(
        gene_type: String,
        codec: Py<PyAny>,
        problem: Py<PyAny>,
        subscribers: Vec<PySubscriber>,
        inputs: Vec<PyEngineInput>,
    ) -> Self {
        let gene_type = match gene_type.as_str() {
            "float" => PyGeneType::Float,
            "int" => PyGeneType::Int,
            "bit" => PyGeneType::Bit,
            "char" => PyGeneType::Char,
            "graph" => PyGeneType::Graph,
            "tree" => PyGeneType::Tree,
            "permutation" => PyGeneType::Permutation,
            _ => panic!("Invalid gene type: {}", gene_type),
        };
        PyEngineBuilder {
            gene_type,
            codec,
            problem,
            subscribers,
            inputs,
        }
    }

    pub fn build<'py>(&mut self, py: Python<'py>) -> PyResult<PyEngine> {
        let mut inner = self.create_builder(py)?;

        let mut accum = HashMap::<PyEngineInputType, Vec<PyEngineInput>>::new();
        let input_groups = self.inputs.iter().fold(&mut accum, |acc, input| {
            acc.entry(input.input_type())
                .or_default()
                .push(input.clone());
            acc
        });

        for (input_type, inputs) in input_groups.iter() {
            inner = self.process_inputs(inner, *input_type, inputs)?;
        }

        inner = self.process_subscriber(inner)?;

        let engine_handle = match inner {
            EngineBuilderHandle::Int(builder) => EngineHandle::Int(builder.build()),
            EngineBuilderHandle::Float(builder) => EngineHandle::Float(builder.build()),
            EngineBuilderHandle::Char(builder) => EngineHandle::Char(builder.build()),
            EngineBuilderHandle::Bit(builder) => EngineHandle::Bit(builder.build()),
            EngineBuilderHandle::Permutation(builder) => EngineHandle::Permutation(builder.build()),
            EngineBuilderHandle::Graph(builder) => EngineHandle::Graph(builder.build()),
            EngineBuilderHandle::Tree(builder) => EngineHandle::Tree(builder.build()),
            _ => {
                return Err(PyTypeError::new_err(
                    "Unsupported builder type for engine creation",
                ));
            }
        };

        Ok(PyEngine::new(engine_handle))
    }

    pub fn __repr__(&self) -> String {
        let inputs = self
            .inputs
            .iter()
            .map(|i| i.__repr__())
            .collect::<Vec<String>>();
        format!(
            "EngineBuilder(\ngene_type={:?}, \ncodec={:?}, \nproblem={:?}, \nsubscribers={:?}, \ninputs=[{}])",
            self.gene_type,
            self.codec,
            self.problem,
            self.subscribers,
            inputs.join(", ")
        )
    }
}

impl PyEngineBuilder {
    fn process_subscriber(&mut self, inner: EngineBuilderHandle) -> PyResult<EngineBuilderHandle> {
        if self.subscribers.is_empty() {
            return Ok(inner);
        }

        let py_subscriber = PyEventHandler::new(self.subscribers.clone());
        apply_to_builder!(inner, subscribe(py_subscriber))
    }

    fn process_inputs(
        &self,
        builder: EngineBuilderHandle,
        input_type: PyEngineInputType,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        match input_type {
            PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector => {
                self.process_selector(builder, inputs)
            }
            PyEngineInputType::Alterer => self.process_alterers(builder, inputs),
            PyEngineInputType::Objective => self.process_objective(builder, inputs),
            PyEngineInputType::MaxPhenotypeAge => self.process_max_phenotype_age(builder, inputs),
            PyEngineInputType::PopulationSize => self.process_population_size(builder, inputs),
            PyEngineInputType::MaxSpeciesAge => self.process_max_species_age(builder, inputs),
            PyEngineInputType::SpeciesThreshold => self.process_species_threshold(builder, inputs),
            PyEngineInputType::OffspringFraction => {
                self.process_offspring_fraction(builder, inputs)
            }
            PyEngineInputType::FrontRange => self.process_front_range(builder, inputs),
            PyEngineInputType::Diversity => self.process_diversity(builder, inputs),
            _ => Ok(builder),
        }
    }

    fn process_single_value<F>(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
        processor: F,
    ) -> PyResult<EngineBuilderHandle>
    where
        F: Fn(EngineBuilderHandle, &PyEngineInput) -> PyResult<EngineBuilderHandle>,
    {
        if inputs.len() > 1 {
            processor(builder, &inputs[inputs.len() - 1])
        } else if inputs.len() == 1 {
            processor(builder, &inputs[0])
        } else {
            Err(PyTypeError::new_err(
                "Expected exactly one input for this configuration",
            ))
        }
    }

    fn process_max_species_age(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let max_age = input.get_usize("age").unwrap_or(usize::MAX);
            apply_to_builder!(builder, max_species_age(max_age))
        })
    }

    fn process_species_threshold(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let threshold = input.get_f32("threshold").unwrap_or(0.3);
            apply_to_builder!(builder, species_threshold(threshold))
        })
    }

    fn process_offspring_fraction(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let fraction = input.get_f32("fraction").unwrap_or(0.8);
            apply_to_builder!(builder, offspring_fraction(fraction))
        })
    }

    fn process_population_size(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let size = input.get_usize("size").unwrap_or(5);
            apply_to_builder!(builder, population_size(size))
        })
    }

    fn process_max_phenotype_age(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let max_age = input.get_usize("age").unwrap_or(usize::MAX);
            apply_to_builder!(builder, max_age(max_age))
        })
    }

    fn process_selector(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| match builder {
            EngineBuilderHandle::Int(builder) => {
                let selector: Box<dyn Select<IntChromosome<i32>>> = input.convert();
                match input.input_type() {
                    PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Int(
                        builder.boxed_survivor_selector(selector),
                    )),
                    PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Int(
                        builder.boxed_offspring_selector(selector),
                    )),
                    _ => Err(PyTypeError::new_err(
                        "process_selector only implemented for Survivor and Offspring selectors",
                    )),
                }
            }
            EngineBuilderHandle::Float(builder) => {
                let selector: Box<dyn Select<FloatChromosome>> = input.convert();
                match input.input_type() {
                    PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Float(
                        builder.boxed_survivor_selector(selector),
                    )),
                    PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Float(
                        builder.boxed_offspring_selector(selector),
                    )),
                    _ => Err(PyTypeError::new_err(
                        "process_selector only implemented for Survivor and Offspring selectors",
                    )),
                }
            }
            EngineBuilderHandle::Char(builder) => {
                let selector: Box<dyn Select<CharChromosome>> = input.convert();
                match input.input_type() {
                    PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Char(
                        builder.boxed_survivor_selector(selector),
                    )),
                    PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Char(
                        builder.boxed_offspring_selector(selector),
                    )),
                    _ => Err(PyTypeError::new_err(
                        "process_selector only implemented for Survivor and Offspring selectors",
                    )),
                }
            }
            EngineBuilderHandle::Bit(builder) => {
                let selector: Box<dyn Select<BitChromosome>> = input.convert();
                match input.input_type() {
                    PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Bit(
                        builder.boxed_survivor_selector(selector),
                    )),
                    PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Bit(
                        builder.boxed_offspring_selector(selector),
                    )),
                    _ => Err(PyTypeError::new_err(
                        "process_selector only implemented for Survivor and Offspring selectors",
                    )),
                }
            }
            EngineBuilderHandle::Permutation(builder) => {
                let selector: Box<dyn Select<PermutationChromosome<usize>>> = input.convert();
                match input.input_type() {
                    PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Permutation(
                        builder.boxed_survivor_selector(selector),
                    )),
                    PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Permutation(
                        builder.boxed_offspring_selector(selector),
                    )),
                    _ => Err(PyTypeError::new_err(
                        "process_selector only implemented for Survivor and Offspring selectors",
                    )),
                }
            }
            EngineBuilderHandle::Graph(builder) => {
                let selector: Box<dyn Select<GraphChromosome<Op<f32>>>> = input.convert();
                match input.input_type() {
                    PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Graph(
                        builder.boxed_survivor_selector(selector),
                    )),
                    PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Graph(
                        builder.boxed_offspring_selector(selector),
                    )),
                    _ => Err(PyTypeError::new_err(
                        "process_selector only implemented for Survivor and Offspring selectors",
                    )),
                }
            }
            EngineBuilderHandle::Tree(builder) => {
                let selector: Box<dyn Select<TreeChromosome<Op<f32>>>> = input.convert();
                match input.input_type() {
                    PyEngineInputType::SurvivorSelector => Ok(EngineBuilderHandle::Tree(
                        builder.boxed_survivor_selector(selector),
                    )),
                    PyEngineInputType::OffspringSelector => Ok(EngineBuilderHandle::Tree(
                        builder.boxed_offspring_selector(selector),
                    )),
                    _ => Err(PyTypeError::new_err(
                        "process_selector only implemented for Survivor and Offspring selectors",
                    )),
                }
            }
            _ => Err(PyTypeError::new_err(
                "process_alterer only implemented for Int gene type",
            )),
        })
    }

    fn process_alterers(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        match builder {
            EngineBuilderHandle::Int(builder) => {
                let alters: Vec<Box<dyn Alter<IntChromosome<i32>>>> = inputs.convert();
                Ok(EngineBuilderHandle::Int(builder.alter(alters)))
            }
            EngineBuilderHandle::Float(builder) => {
                let alters: Vec<Box<dyn Alter<FloatChromosome>>> = inputs.convert();
                Ok(EngineBuilderHandle::Float(builder.alter(alters)))
            }
            EngineBuilderHandle::Char(builder) => {
                let alters: Vec<Box<dyn Alter<CharChromosome>>> = inputs.convert();
                Ok(EngineBuilderHandle::Char(builder.alter(alters)))
            }
            EngineBuilderHandle::Bit(builder) => {
                let alters: Vec<Box<dyn Alter<BitChromosome>>> = inputs.convert();
                Ok(EngineBuilderHandle::Bit(builder.alter(alters)))
            }
            EngineBuilderHandle::Graph(builder) => {
                let alters: Vec<Box<dyn Alter<GraphChromosome<Op<f32>>>>> = inputs.convert();
                Ok(EngineBuilderHandle::Graph(builder.alter(alters)))
            }
            EngineBuilderHandle::Tree(builder) => {
                let alters: Vec<Box<dyn Alter<TreeChromosome<Op<f32>>>>> = inputs.convert();
                Ok(EngineBuilderHandle::Tree(builder.alter(alters)))
            }
            EngineBuilderHandle::Permutation(builder) => {
                let alters: Vec<Box<dyn Alter<PermutationChromosome<usize>>>> = inputs.convert();
                Ok(EngineBuilderHandle::Permutation(builder.alter(alters)))
            }
            _ => Err(PyTypeError::new_err(format!(
                "Process Alterer not imiplemented for {:?} gene type",
                self.gene_type
            ))),
        }
    }

    fn process_diversity(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            match builder {
                EngineBuilderHandle::Int(b) => {
                    let diversity: Option<Box<dyn Diversity<IntChromosome<i32>>>> = input.convert();
                    Ok(EngineBuilderHandle::Int(b.boxed_diversity(diversity)))
                }
                EngineBuilderHandle::Float(b) => {
                    let diversity: Option<Box<dyn Diversity<FloatChromosome>>> = input.convert();
                    Ok(EngineBuilderHandle::Float(b.boxed_diversity(diversity)))
                }
                EngineBuilderHandle::Char(b) => {
                    let diversity: Option<Box<dyn Diversity<CharChromosome>>> = input.convert();
                    Ok(EngineBuilderHandle::Char(b.boxed_diversity(diversity)))
                }
                EngineBuilderHandle::Bit(b) => {
                    let diversity: Option<Box<dyn Diversity<BitChromosome>>> = input.convert();
                    Ok(EngineBuilderHandle::Bit(b.boxed_diversity(diversity)))
                }
                EngineBuilderHandle::Graph(b) => {
                    let diversity: Option<Box<dyn Diversity<GraphChromosome<Op<f32>>>>> = input.convert();
                    Ok(EngineBuilderHandle::Graph(b.boxed_diversity(diversity)))
                }
                EngineBuilderHandle::Permutation(b) => {
                    let diversity: Option<Box<dyn Diversity<PermutationChromosome<usize>>>> = input.convert();
                    Ok(EngineBuilderHandle::Permutation(b.boxed_diversity(diversity)))
                }
                _ => Err(PyTypeError::new_err(
                    "process_diversity only implemented for Int, Float, Char, Bit, and Graph gene types",
                )),
            }
        })
    }

    fn process_objective(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let objectives = input.get_string("objective").map(|objs| {
                objs.split('|')
                    .map(|s| match s.trim().to_lowercase().as_str() {
                        "min" => Optimize::Minimize,
                        "max" => Optimize::Maximize,
                        _ => panic!("Objective {} not recognized", s),
                    })
                    .collect::<Vec<Optimize>>()
            });

            let opt = match objectives {
                Some(objs) => {
                    if objs.len() == 1 {
                        Objective::Single(objs[0])
                    } else if objs.len() > 1 {
                        Objective::Multi(objs)
                    } else {
                        panic!("No objectives provided");
                    }
                }
                None => Objective::Single(Optimize::Maximize),
            };

            match opt {
                Objective::Single(opt) => match opt {
                    Optimize::Minimize => apply_to_builder!(builder, minimizing()),
                    Optimize::Maximize => apply_to_builder!(builder, maximizing()),
                },
                Objective::Multi(opts) => apply_to_builder!(builder, multi_objective(opts)),
            }
        })
    }

    fn process_front_range(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let min = input.get_usize("min").unwrap_or(800);
            let max = input.get_usize("max").unwrap_or(1000);

            if min > max {
                return Err(PyTypeError::new_err(
                    "Minimum size cannot be greater than maximum size",
                ));
            }

            if min == 0 || max == 0 {
                return Err(PyTypeError::new_err(
                    "Minimum and maximum size must be greater than zero",
                ));
            }

            apply_to_builder!(builder, front_size(min..max))
        })
    }

    fn create_builder<'py>(&self, py: Python<'py>) -> PyResult<EngineBuilderHandle> {
        let problem = self.problem.bind(py).extract::<PyProblemBuilder>()?;
        let codec = self.codec.bind(py);

        let executor = self
            .inputs
            .iter()
            .filter(|i| i.input_type == PyEngineInputType::Executor)
            .filter_map(|input| input.clone().into())
            .next()
            .unwrap_or(Executor::Serial);

        let builder = if problem.name() == "Custom" {
            let fitness_fn = problem
                .args(py)?
                .get_item("fitness_func")?
                .extract::<Py<PyAny>>()?;

            match self.gene_type {
                PyGeneType::Float => {
                    if let Ok(codec) = codec.extract::<PyFloatCodec>() {
                        let float_problem = PyProblem::new(fitness_fn, codec.codec);
                        Ok(EngineBuilderHandle::Float(
                            GeneticEngine::builder()
                                .problem(float_problem.clone())
                                .executor(executor.clone())
                                .evaluator(FreeThreadPyEvaluator::new(executor, float_problem))
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyFloatCodec for gene_type Float",
                        ))
                    }
                }
                PyGeneType::Int => {
                    if let Ok(codec) = codec.extract::<PyIntCodec>() {
                        let int_problem = PyProblem::new(fitness_fn, codec.codec);
                        Ok(EngineBuilderHandle::Int(
                            GeneticEngine::builder()
                                .problem(int_problem.clone())
                                .executor(executor.clone())
                                .evaluator(FreeThreadPyEvaluator::new(executor, int_problem))
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyIntCodec for gene_type Int",
                        ))
                    }
                }
                PyGeneType::Char => {
                    if let Ok(codec) = codec.extract::<PyCharCodec>() {
                        let char_problem = PyProblem::new(fitness_fn, codec.codec);
                        Ok(EngineBuilderHandle::Char(
                            GeneticEngine::builder()
                                .problem(char_problem.clone())
                                .executor(executor.clone())
                                .evaluator(FreeThreadPyEvaluator::new(executor, char_problem))
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyCharCodec for gene_type Char",
                        ))
                    }
                }
                PyGeneType::Bit => {
                    if let Ok(codec) = codec.extract::<PyBitCodec>() {
                        let bit_problem = PyProblem::new(fitness_fn, codec.codec);

                        Ok(EngineBuilderHandle::Bit(
                            GeneticEngine::builder()
                                .problem(bit_problem.clone())
                                .executor(executor.clone())
                                .evaluator(FreeThreadPyEvaluator::new(executor, bit_problem))
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyBitCodec for gene_type Bit",
                        ))
                    }
                }
                PyGeneType::Graph => {
                    if let Ok(codec) = codec.extract::<PyGraphCodec>() {
                        let cloned_codec = codec.codec.clone();
                        let py_codec = PyCodec::new()
                            .with_encoder(move || cloned_codec.encode())
                            .with_decoder(move |_, genotype| codec.codec.decode(genotype));

                        let graph_problem = PyProblem::new(fitness_fn, py_codec);
                        Ok(EngineBuilderHandle::Graph(
                            GeneticEngine::builder()
                                .problem(graph_problem.clone())
                                .executor(executor.clone())
                                .evaluator(FreeThreadPyEvaluator::new(executor, graph_problem))
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyGraphCodec for gene_type Graph",
                        ))
                    }
                }
                PyGeneType::Tree => {
                    if let Ok(codec) = codec.extract::<PyTreeCodec>() {
                        let cloned_codec = codec.codec.clone();
                        let py_codec = PyCodec::new()
                            .with_encoder(move || cloned_codec.encode())
                            .with_decoder(move |_, genotype| codec.codec.decode(genotype));

                        let tree_problem = PyProblem::new(fitness_fn, py_codec);
                        Ok(EngineBuilderHandle::Tree(
                            GeneticEngine::builder()
                                .problem(tree_problem.clone())
                                .executor(executor.clone())
                                .evaluator(FreeThreadPyEvaluator::new(executor, tree_problem))
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyTreeCodec for gene_type Tree",
                        ))
                    }
                }
                PyGeneType::Permutation => {
                    if let Ok(codec) = codec.extract::<PyPermutationCodec>() {
                        let cloned_codec = codec.codec.clone();
                        let py_codec = PyCodec::new()
                            .with_encoder(move || cloned_codec.encode())
                            .with_decoder(move |_, genotype| codec.codec.decode(genotype));

                        let custom_problem = PyProblem::new(fitness_fn, py_codec);
                        Ok(EngineBuilderHandle::Permutation(
                            GeneticEngine::builder()
                                .problem(custom_problem.clone())
                                .executor(executor.clone())
                                .evaluator(FreeThreadPyEvaluator::new(executor, custom_problem))
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyPermutationCodec for gene_type Permutation",
                        ))
                    }
                }
                _ => Err(PyTypeError::new_err(format!(
                    "Unsupported gene_type {:?} for Custom problem",
                    self.gene_type
                ))),
            }
        } else if problem.name() == "Regression" {
            let features = problem
                .args(py)
                .and_then(|args| args.get_item("features"))
                .and_then(|f| f.extract::<Vec<Vec<f32>>>())?;
            let targets = problem
                .args(py)
                .and_then(|args| args.get_item("targets"))
                .and_then(|t| t.extract::<Vec<Vec<f32>>>())?;
            let loss_str = problem
                .args(py)
                .and_then(|args| args.get_item("loss"))
                .and_then(|l| l.extract::<String>())
                .map(|s| s.to_uppercase())
                .unwrap_or("MSE".into());
            let loss = match loss_str.as_str() {
                "MSE" => Loss::MSE,
                "MAE" => Loss::MAE,
                "CROSS_ENTROPY" => Loss::CrossEntropy,
                _ => {
                    return Err(PyErr::new::<PyTypeError, _>(
                        "Unsupported loss function for regression",
                    ));
                }
            };

            let data_set = DataSet::new(features, targets);

            match self.gene_type {
                PyGeneType::Graph => {
                    if let Ok(codec) = codec.extract::<PyGraphCodec>() {
                        let regression = Regression::new(data_set, loss);

                        Ok(EngineBuilderHandle::Graph(
                            GeneticEngine::builder()
                                .codec(codec.codec.clone())
                                .fitness_fn(regression)
                                .executor(executor)
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyGraphCodec for gene_type Graph",
                        ))
                    }
                }
                PyGeneType::Tree => {
                    if let Ok(codec) = codec.extract::<PyTreeCodec>() {
                        let regression = Regression::new(data_set, loss);

                        Ok(EngineBuilderHandle::Tree(
                            GeneticEngine::builder()
                                .codec(codec.codec.clone())
                                .fitness_fn(regression)
                                .executor(executor)
                                .bus_executor(Executor::default()),
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyTreeCodec for gene_type Tree",
                        ))
                    }
                }
                _ => {
                    return Err(PyErr::new::<PyTypeError, _>(
                        "Regression problem only supports GraphChromosome",
                    ));
                }
            }
        } else {
            return Err(PyErr::new::<PyTypeError, _>(
                "Unsupported problem type. Only 'DefaultProblem' and 'Regression' are supported",
            ));
        };

        builder
    }
}
