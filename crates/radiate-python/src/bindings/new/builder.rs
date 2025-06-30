use crate::bindings::new::{EngineBuilderHandle, EngineHandle};
use crate::{prelude::*, FreeThreadPyEvaluator, InputConverter, PyEngine, PyEngineInput, PyEngineInputType};
use crate::{PyGeneType};
use core::panic;
use pyo3::exceptions::PyTypeError;
use pyo3::{ Py, PyAny, pyclass, pymethods, types::PyAnyMethods};
use radiate::prelude::*;
use std::collections::{HashMap};

macro_rules! apply_to_builder {
    ($builder:expr, $method:ident($($args:expr),*)) => {
        match $builder {
            EngineBuilderHandle::Int(b) => Ok(EngineBuilderHandle::Int(b.$method($($args),*))),
            EngineBuilderHandle::Float(b) => Ok(EngineBuilderHandle::Float(b.$method($($args),*))),
            EngineBuilderHandle::Char(b) => Ok(EngineBuilderHandle::Char(b.$method($($args),*))),
            EngineBuilderHandle::Bit(b) => Ok(EngineBuilderHandle::Bit(b.$method($($args),*))),
            EngineBuilderHandle::IntMulti(b) => Ok(EngineBuilderHandle::IntMulti(b.$method($($args),*))),
            EngineBuilderHandle::FloatMulti(b) => Ok(EngineBuilderHandle::FloatMulti(b.$method($($args),*))),
            EngineBuilderHandle::GraphRegression(b) => Ok(EngineBuilderHandle::GraphRegression(b.$method($($args),*))),
            EngineBuilderHandle::Empty => Err(PyTypeError::new_err(
                "EngineBuilder must have a problem and codec before processing other inputs",
            )),
        }
    };
}

macro_rules! apply_selector_to_builder {
    ($builder:expr, $selector:expr, $method:ident) => {
        match $builder {
            EngineBuilderHandle::Int(b) => {
                let selector: Box<dyn Select<IntChromosome<i32>>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderHandle::Int(b.$method(selector)))
            }
            EngineBuilderHandle::Float(b) => {
                let selector: Box<dyn Select<FloatChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderHandle::Float(b.$method(selector)))
            }
            EngineBuilderHandle::Char(b) => {
                let selector: Box<dyn Select<CharChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderHandle::Char(b.$method(selector)))
            }
            EngineBuilderHandle::Bit(b) => {
                let selector: Box<dyn Select<BitChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderHandle::Bit(b.$method(selector)))
            }
            EngineBuilderHandle::IntMulti(b) => {
                let selector: Box<dyn Select<IntChromosome<i32>>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderHandle::IntMulti(b.$method(selector)))
            }
            EngineBuilderHandle::FloatMulti(b) => {
                let selector: Box<dyn Select<FloatChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderHandle::FloatMulti(b.$method(selector)))
            }
            EngineBuilderHandle::GraphRegression(b) => {
                let selector: Box<dyn Select<GraphChromosome<Op<f32>>>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderHandle::GraphRegression(b.$method(selector)))
            }
            EngineBuilderHandle::Empty => Err(PyTypeError::new_err(
                "EngineBuilder must have a problem and codec before processing other inputs",
            )),
        }
    };
}



#[pyclass]
#[derive(Debug)]
pub struct PyEngineBuilder {
    pub gene_type: PyGeneType,
    pub codec: Py<PyAny>,
    pub problem: Py<PyAny>,
    pub inputs: Vec<PyEngineInput>,
}

#[pymethods]
impl PyEngineBuilder {
    #[new]
    pub fn new(
        gene_type: String,
        codec: Py<PyAny>,
        problem: Py<PyAny>,
        inputs: Vec<PyEngineInput>,
    ) -> Self {
        let gene_type = match gene_type.as_str() {
            "float" => PyGeneType::Float,
            "int" => PyGeneType::Int,
            "bit" => PyGeneType::Bit,
            "char" => PyGeneType::Char,
            "graph" => PyGeneType::Graph,
            _ => panic!("Invalid gene type: {}", gene_type),
        };
        PyEngineBuilder {
            gene_type,
            codec,
            problem,
            inputs,
        }
    }

    pub fn build<'py>(&mut self, py: Python<'py>) -> PyResult<PyEngine> {
        
        let mut inner = self.create_builder(py)?;

        let mut accum = HashMap::<PyEngineInputType, Vec<PyEngineInput>>::new();
        let input_groups = self.inputs.iter().fold(
            &mut accum,
            |acc, input| {
                acc.entry(input.input_type()).or_default().push(input.clone());
                acc
            },
        );

        for (input_type, inputs) in input_groups.iter() {
            inner = self.process_inputs(inner, *input_type, inputs)?;
        }

        let engine_handle = match inner {
            EngineBuilderHandle::Int(builder) => EngineHandle::Int(builder.build()),
            EngineBuilderHandle::Float(builder) => EngineHandle::Float(builder.build()),
            EngineBuilderHandle::Char(builder) => EngineHandle::Char(builder.build()),
            EngineBuilderHandle::Bit(builder) => EngineHandle::Bit(builder.build()),
            EngineBuilderHandle::IntMulti(builder) => EngineHandle::IntMulti(builder.build()),
            EngineBuilderHandle::FloatMulti(builder) => EngineHandle::FloatMulti(builder.build()),
            EngineBuilderHandle::GraphRegression(builder) => {
                EngineHandle::GraphRegression(builder.build())
            }
            _ => {
                return Err(PyTypeError::new_err(
                    "Unsupported builder type for engine creation",
                ));
            }
        };

        Ok(PyEngine::new(engine_handle))
    }
}

impl PyEngineBuilder {

    fn create_builder<'py>(&self, py: Python<'py>) -> PyResult<EngineBuilderHandle> {
        
        let problem = self
            .problem
            .bind(py)
            .extract::<PyProblemBuilder>()?;
        
        let codec = self.codec.bind(py);

        let input = self.inputs
            .iter()
            .filter(|i| i.input_type == PyEngineInputType::Executor)
            .next();

        let executor = if let Some(input) = input {
            match input.component.as_str() {
                "Serial" => Executor::Serial,
                "FixedSizedWorkerPool" => {
                    let num_workers = input
                        .args()
                        .get("num_workers")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1);

                    Executor::FixedSizedWorkerPool(num_workers)
                }
                "WorkerPool" => Executor::WorkerPool,
                _ => panic!("Executor type {} not yet implemented", input.component),
            }
        } else {
            Executor::Serial
        };
        
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
                                .bus_executor(Executor::default())
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
                                .bus_executor(Executor::default())
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
                              .bus_executor(Executor::default())
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
                                .bus_executor(Executor::default())
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyBitCodec for gene_type Bit",
                        ))
                    }
                }
                _ => {
                    Err(PyTypeError::new_err(format!(
                        "Unsupported gene_type {:?} for Custom problem",
                        self.gene_type
                    )))
                }
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

            println!("Features: {:?}", features);
            println!("Targets: {:?}", targets);

            let data_set = DataSet::new(features, targets);

            match self.gene_type {
                PyGeneType::Graph => {
                    if let Ok(codec) = codec.extract::<PyGraphCodec>() {
                        let loss_str = problem
                            .args(py)
                            .and_then(|args| args.get_item("loss"))
                            .and_then(|l| l.extract::<String>())
                            .map(|s| s.to_uppercase())
                            .unwrap_or("MSE".into());
                        let loss = match loss_str.as_str() {
                            "MSE" => Loss::MSE,
                            "MAE" => Loss::MAE,
                            _ => {
                                return Err(PyErr::new::<PyTypeError, _>(
                                    "Unsupported loss function for regression",
                                ));
                            }
                        };

                        let regression = Regression::new(data_set, loss, codec.codec);

                        println!("{:?}", regression.encode());

                         Ok(EngineBuilderHandle::GraphRegression(
                            GeneticEngine::builder()
                                .problem(regression)
                                .executor(executor)
                                .bus_executor(Executor::default())
                        ))
                    } else {
                        Err(PyTypeError::new_err(
                            "Expected a PyGraphCodec for gene_type Graph",
                        ))
                    }
                }
                _ => return Err(PyErr::new::<PyTypeError, _>(
                    "Regression problem only supports GraphChromosome",
                )),
            }
        } else {
            return Err(PyErr::new::<PyTypeError, _>(
                "Unsupported problem type. Only 'DefaultProblem' and 'Regression' are supported",
            ));
        };

        builder
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
            // PyEngineInputType::Executor => self.process_executor(builder, inputs),
            // PyEngineInputType::Evaluator => self.proce(builder, inputs), // Add this
            PyEngineInputType::MaxPhenotypeAge => self.process_max_phenotype_age(builder, inputs),
            PyEngineInputType::PopulationSize => self.process_population_size(builder, inputs),
            PyEngineInputType::MaxSpeciesAge => self.process_max_species_age(builder, inputs),
            PyEngineInputType::SpeciesThreshold => self.process_species_threshold(builder, inputs),
            PyEngineInputType::OffspringFraction => self.process_offspring_fraction(builder, inputs),
            PyEngineInputType::FrontRange => self.process_front_range(builder, inputs),
            // PyEngineInputType::Diversity => self.process_diversity(builder, inputs),
            // PyEngineInputType::Limit => self.process_limit(builder, inputs),
            // PyEngineInputType::Subscriber => self.process_subscriber(builder, inputs),
            PyEngineInputType::Codec | PyEngineInputType::Problem => {
                // These are handled separately in the build method
                Ok(builder)
            }
            _ => Ok(builder),
            // _ => Err(PyTypeError::new_err(format!(
            //     "Input type {:?} not yet implemented",
            //     input_type
            // ))),
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
        if inputs.len() != 1 {
            return Err(PyTypeError::new_err(
                "Only one input of this type can be processed at a time",
            ));
        }
        processor(builder, &inputs[0])
    }


    fn process_max_species_age(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let max_age = input
                .args
                .get("max_age")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(usize::MAX);

            apply_to_builder!(builder, max_species_age(max_age))
        })
    }

    fn process_species_threshold(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let threshold = input
                .args
                .get("threshold")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.1);

            apply_to_builder!(builder, species_threshold(threshold))
        })
    }

    fn process_offspring_fraction(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let fraction = input
                .args
                .get("fraction")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.5);

            apply_to_builder!(builder, offspring_fraction(fraction))
        })
    }

    fn process_population_size(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let size = input
                .args
                .get("size")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100);

            apply_to_builder!(builder, population_size(size))
        })
    }

    fn process_selector(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let selector: Box<dyn Select<FloatChromosome> + 'static> = match builder {
                EngineBuilderHandle::Int(_) => input.convert(),
                EngineBuilderHandle::Float(_) => input.convert(),
                EngineBuilderHandle::Char(_) => input.convert(),
                EngineBuilderHandle::Bit(_) => input.convert(),
                EngineBuilderHandle::IntMulti(_) => input.convert(),
                EngineBuilderHandle::FloatMulti(_) => input.convert(),
                EngineBuilderHandle::GraphRegression(_) => input.convert(),
                EngineBuilderHandle::Empty => {
                    return Err(PyTypeError::new_err("EngineBuilder must have a problem and codec before processing other inputs"));
                }
            };

            
            match input.input_type() {
                PyEngineInputType::SurvivorSelector => {
                    apply_selector_to_builder!(builder, selector, boxed_survivor_selector)
                }
                PyEngineInputType::OffspringSelector => {
                    apply_selector_to_builder!(builder, selector, boxed_offspring_selector)
                }
                _ => {
                    Err(PyTypeError::new_err("process_selector only implemented for Survivor and Offspring selectors"))
                }
            }
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
            EngineBuilderHandle::IntMulti(builder) => {
                let alters: Vec<Box<dyn Alter<IntChromosome<i32>>>> = inputs.convert();
                Ok(EngineBuilderHandle::IntMulti(builder.alter(alters)))
            }
            EngineBuilderHandle::FloatMulti(builder) => {
                let alters: Vec<Box<dyn Alter<FloatChromosome>>> = inputs.convert();
                Ok(EngineBuilderHandle::FloatMulti(builder.alter(alters)))
            }
            EngineBuilderHandle::GraphRegression(builder) => {
                let alters: Vec<Box<dyn Alter<GraphChromosome<Op<f32>>>>> = inputs.convert();
                Ok(EngineBuilderHandle::GraphRegression(builder.alter(alters)))
            }
            _ => Err(PyTypeError::new_err(
                "process_alterer only implemented for Int gene type",
            )),
        }
    }

    fn process_executor(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let executor = match input.component.as_str() {
                "Serial" => Executor::Serial,
                "FixedSizedWorkerPool" => {
                    let num_workers = input
                        .args()
                        .get("num_workers")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                    
                    Executor::FixedSizedWorkerPool(num_workers)
                }
                "WorkerPool" => Executor::WorkerPool,
                _ => panic!("Executor type {} not yet implemented", input.component),
            };

            apply_to_builder!(builder, executor(executor))
                .and_then(|builder| {
                    apply_to_builder!(builder, bus_executor(Executor::default()))
                })
        })
    }

    fn process_max_phenotype_age(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let max_age = input
                .args
                .get("max_age")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(usize::MAX);

            apply_to_builder!(builder, max_age(max_age))
        })
    }

    fn process_objective(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let objectives = input.args().get("objective").map(|objs| {
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
                Objective::Single(opt) => {
                    match opt {
                        Optimize::Minimize => apply_to_builder!(builder, minimizing()),
                        Optimize::Maximize => apply_to_builder!(builder, maximizing()),
                    }
                }
                Objective::Multi(opts) => {
                    match builder {
                        EngineBuilderHandle::Int(inner) => {
                            Ok(EngineBuilderHandle::IntMulti(inner.multi_objective(opts)))
                        }
                        EngineBuilderHandle::Float(inner) => {
                            Ok(EngineBuilderHandle::FloatMulti(inner.multi_objective(opts)))
                        }
                        _ => Err(PyTypeError::new_err(
                            "Multi-objective only implemented for Int and Float gene types",
                        )),
                    }
                }
            }
        })
    }

    fn process_front_range(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, inputs, |builder, input| {
            let min = input
                .args
                .get("min")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(800);

            let max = input
                .args
                .get("max")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1000);

            match builder {
                EngineBuilderHandle::Int(b) => {
                    Ok(EngineBuilderHandle::IntMulti(b.front_size(min..max)))
                }
                EngineBuilderHandle::Float(b) => {
                    Ok(EngineBuilderHandle::FloatMulti(b.front_size(min..max)))
                }
                EngineBuilderHandle::Char(_) => {
                    Err(PyTypeError::new_err(
                        "process_front_range not implemented for Char gene type",
                    ))
                }
                EngineBuilderHandle::Bit(_) => {
                    Err(PyTypeError::new_err(
                        "process_front_range not implemented for Bit gene type",
                    ))
                }
                EngineBuilderHandle::IntMulti(b) => {
                    Ok(EngineBuilderHandle::IntMulti(b.front_size(min..max)))
                }
                EngineBuilderHandle::FloatMulti(b) => {
                    Ok(EngineBuilderHandle::FloatMulti(b.front_size(min..max)))
                }
                EngineBuilderHandle::GraphRegression(_) => {
                    Err(PyTypeError::new_err(
                        "process_front_range not implemented for Graph gene type",
                    ))
                }
                _ => Err(PyTypeError::new_err(
                    "process_front_range only implemented for single and multi objective Int, Float, Char, and Bit gene types",
                )),
            }
        })
    }
}

