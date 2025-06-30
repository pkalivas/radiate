use crate::bindings::new::{EngineBuilderHandle, EngineHandle};
use crate::{prelude::*, InputConverter, PyEngineInput, PyEngineInputType, PyEngine};
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
            EngineBuilderHandle::GraphRegression(_) => Err(PyTypeError::new_err(
                "process_selector not implemented for Graph gene type",
            )),
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
        let mut inner = EngineBuilderHandle::Empty;
        let problem = self
            .problem
            .bind(py)
            .extract::<PyProblemBuilder>()?;
        let codec = self.codec.bind(py);

        if problem.name() == "Custom" {
            let fitness_fn = problem
                .args(py)?
                .get_item("fitness_func")?
                .extract::<Py<PyAny>>()?;

            match self.gene_type {
                PyGeneType::Float => {
                    if let Ok(codec) = codec.extract::<PyFloatCodec>() {
                        let float_problem = PyProblem::new(fitness_fn, codec.codec);
                        let builder = GeneticEngine::builder().problem(float_problem);
                        inner = EngineBuilderHandle::Float(builder);
                    } else {
                        return Err(PyTypeError::new_err(
                            "Expected a PyFloatCodec for gene_type Float",
                        ));
                    }
                }
                PyGeneType::Int => {
                    if let Ok(codec) = codec.extract::<PyIntCodec>() {
                        let int_problem = PyProblem::new(fitness_fn, codec.codec);
                        let builder = GeneticEngine::builder().problem(int_problem);
                        inner = EngineBuilderHandle::Int(builder);
                    } else {
                        return Err(PyTypeError::new_err(
                            "Expected a PyIntCodec for gene_type Int",
                        ));
                    }
                }
                _ => {
                    return Err(PyTypeError::new_err(format!(
                        "Unsupported gene_type {:?} for Custom problem",
                        self.gene_type
                    )));
                }
            }
        }

        let input_groups = self.inputs.iter().fold(
            HashMap::<PyEngineInputType, Vec<&PyEngineInput>>::new(),
            |mut acc, input| {
                acc.entry(input.input_type()).or_default().push(input);
                acc
            },
        );

        for (input_type, inputs) in input_groups.iter() {
            inner = match *input_type {
                PyEngineInputType::SurvivorSelector => self.process_selector(inner, inputs)?,
                PyEngineInputType::OffspringSelector => self.process_selector(inner, inputs)?,
                PyEngineInputType::Alterer => self.process_alterser(inner, inputs)?,
                PyEngineInputType::Objective => self.process_objective(inner, inputs)?,
                PyEngineInputType::Executor => self.process_executor(inner, inputs)?,
                PyEngineInputType::MaxPhenotypeAge => {
                    self.process_max_phenotype_age(inner, inputs)?
                }
                PyEngineInputType::PopulationSize => self.process_population_size(inner, inputs)?,
                PyEngineInputType::MaxSpeciesAge => self.process_max_species_age(inner, inputs)?,
                PyEngineInputType::SpeciesThreshold => {
                    self.process_species_threshold(inner, inputs)?
                }
                PyEngineInputType::OffspringFraction => {
                    self.process_offspring_fraction(inner, inputs)?
                }
                PyEngineInputType::FrontRange => self.process_front_range(inner, inputs)?,
                _ => {
                    return Err(PyTypeError::new_err(format!(
                        "Input type {:?} not yet implemented",
                        input_type
                    )));
                }
            }
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

    fn process_single_value<F>(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[&PyEngineInput],
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
        processor(builder, inputs[0])
    }


    fn process_max_species_age(
        &self,
        builder: EngineBuilderHandle,
        inputs: &[&PyEngineInput],
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
        inputs: &[&PyEngineInput],
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
        inputs: &[&PyEngineInput],
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
        inputs: &[&PyEngineInput],
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
        inputs: &[&PyEngineInput],
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

    fn process_alterser(
        &self,
        builder: EngineBuilderHandle,
        input: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        match builder {
            EngineBuilderHandle::Int(builder) => {
                let alters: Vec<Box<dyn Alter<IntChromosome<i32>>>> = input.convert();
                Ok(EngineBuilderHandle::Int(builder.alter(alters)))
            }
            EngineBuilderHandle::Float(builder) => {
                let alters: Vec<Box<dyn Alter<FloatChromosome>>> = input.convert();
                Ok(EngineBuilderHandle::Float(builder.alter(alters)))
            }
            EngineBuilderHandle::Char(builder) => {
                let alters: Vec<Box<dyn Alter<CharChromosome>>> = input.convert();
                Ok(EngineBuilderHandle::Char(builder.alter(alters)))
            }
            EngineBuilderHandle::Bit(builder) => {
                let alters: Vec<Box<dyn Alter<BitChromosome>>> = input.convert();
                Ok(EngineBuilderHandle::Bit(builder.alter(alters)))
            }
            EngineBuilderHandle::IntMulti(builder) => {
                let alters: Vec<Box<dyn Alter<IntChromosome<i32>>>> = input.convert();
                Ok(EngineBuilderHandle::IntMulti(builder.alter(alters)))
            }
            EngineBuilderHandle::FloatMulti(builder) => {
                let alters: Vec<Box<dyn Alter<FloatChromosome>>> = input.convert();
                Ok(EngineBuilderHandle::FloatMulti(builder.alter(alters)))
            }
            EngineBuilderHandle::GraphRegression(builder) => {
                let alters: Vec<Box<dyn Alter<GraphChromosome<Op<f32>>>>> = input.convert();
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
        input: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, input, |builder, input| {
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
        input: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderHandle> {
        self.process_single_value(builder, input, |builder, input| {
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
        inputs: &[&PyEngineInput],
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
        inputs: &[&PyEngineInput],
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

