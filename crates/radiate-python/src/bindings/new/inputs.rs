use crate::bindings::codec;
use crate::{InputConverter, prelude::*};
use crate::{PyChromosomeType, PyGeneType};
use core::panic;
use pyo3::exceptions::PyTypeError;
use pyo3::{FromPyObject, Py, PyAny, pyclass, pymethods, types::PyAnyMethods};
use radiate::prelude::*;
use std::collections::{HashMap, HashSet};

macro_rules! apply_to_builder {
    ($builder:expr, $method:ident($($args:expr),*)) => {
        match $builder {
            EngineBuilderInner::Int(b) => Ok(EngineBuilderInner::Int(b.$method($($args),*))),
            EngineBuilderInner::Float(b) => Ok(EngineBuilderInner::Float(b.$method($($args),*))),
            EngineBuilderInner::Char(b) => Ok(EngineBuilderInner::Char(b.$method($($args),*))),
            EngineBuilderInner::Bit(b) => Ok(EngineBuilderInner::Bit(b.$method($($args),*))),
            EngineBuilderInner::IntMulti(b) => Ok(EngineBuilderInner::IntMulti(b.$method($($args),*))),
            EngineBuilderInner::FloatMulti(b) => Ok(EngineBuilderInner::FloatMulti(b.$method($($args),*))),
            EngineBuilderInner::GraphRegression(b) => Ok(EngineBuilderInner::GraphRegression(b.$method($($args),*))),
            EngineBuilderInner::Empty => Err(PyTypeError::new_err(
                "EngineBuilder must have a problem and codec before processing other inputs",
            )),
        }
    };
}

macro_rules! apply_selector_to_builder {
    ($builder:expr, $selector:expr, $method:ident) => {
        match $builder {
            EngineBuilderInner::Int(b) => {
                let selector: Box<dyn Select<IntChromosome<i32>>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderInner::Int(b.$method(selector)))
            }
            EngineBuilderInner::Float(b) => {
                let selector: Box<dyn Select<FloatChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderInner::Float(b.$method(selector)))
            }
            EngineBuilderInner::Char(b) => {
                let selector: Box<dyn Select<CharChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderInner::Char(b.$method(selector)))
            }
            EngineBuilderInner::Bit(b) => {
                let selector: Box<dyn Select<BitChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderInner::Bit(b.$method(selector)))
            }
            EngineBuilderInner::IntMulti(b) => {
                let selector: Box<dyn Select<IntChromosome<i32>>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderInner::IntMulti(b.$method(selector)))
            }
            EngineBuilderInner::FloatMulti(b) => {
                let selector: Box<dyn Select<FloatChromosome>> =
                    unsafe { std::mem::transmute($selector) };
                Ok(EngineBuilderInner::FloatMulti(b.$method(selector)))
            }
            EngineBuilderInner::GraphRegression(_) => Err(PyTypeError::new_err(
                "process_selector not implemented for Graph gene type",
            )),
            EngineBuilderInner::Empty => Err(PyTypeError::new_err(
                "EngineBuilder must have a problem and codec before processing other inputs",
            )),
        }
    };
}

#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum PyEngineInputType {
    Alterer,
    OffspringSelector,
    SurvivorSelector,
    Diversity,
    Objective,
    Limit,
    Subscriber,
    PopulationSize,
    OffspringFraction,
    MaxSpeciesAge,
    MaxPhenotypeAge,
    FrontRange,
    Codec,
    Executor,
    Problem,
    SpeciesThreshold,
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyEngineInput {
    pub component: String,
    pub input_type: PyEngineInputType,
    pub args: HashMap<String, String>,
    pub allowed_genes: HashSet<PyGeneType>,
}

#[pymethods]
impl PyEngineInput {
    #[new]
    pub fn new(
        component: String,
        input_type: PyEngineInputType,
        args: HashMap<String, String>,
        allowed_genes: HashSet<PyGeneType>,
    ) -> Self {
        PyEngineInput {
            component,
            input_type,
            args,
            allowed_genes,
        }
    }

    pub fn component(&self) -> String {
        self.component.clone()
    }

    pub fn input_type(&self) -> PyEngineInputType {
        self.input_type.clone()
    }

    pub fn args(&self) -> HashMap<String, String> {
        self.args.clone()
    }

    pub fn allowed_genes(&self) -> HashSet<PyGeneType> {
        self.allowed_genes.clone()
    }

    pub fn is_valid_gene(&self, gene_type: PyGeneType) -> bool {
        self.allowed_genes.contains(&gene_type)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "EngineParam(component={}, input_type={:?}, args={:?}, allowed_genes={:?})",
            self.component, self.input_type, self.args, self.allowed_genes
        )
    }
    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

type SingleObjBuilder<C, T> = GeneticEngineBuilder<C, T, Generation<C, T>>;
type MultiObjBuilder<C, T> = GeneticEngineBuilder<C, T, ParetoGeneration<C>>;
type RegressionBuilder<C, T> = GeneticEngineBuilder<C, T, Generation<C, T>>;

enum EngineBuilderInner {
    Empty,
    Int(SingleObjBuilder<IntChromosome<i32>, ObjectValue>),
    Float(SingleObjBuilder<FloatChromosome, ObjectValue>),
    Char(SingleObjBuilder<CharChromosome, ObjectValue>),
    Bit(SingleObjBuilder<BitChromosome, ObjectValue>),
    IntMulti(MultiObjBuilder<IntChromosome<i32>, ObjectValue>),
    FloatMulti(MultiObjBuilder<FloatChromosome, ObjectValue>),
    GraphRegression(RegressionBuilder<GraphChromosome<Op<f32>>, Graph<Op<f32>>>),
}

#[pyclass]
#[derive(Debug)]
pub struct PyEngineBuilderTwo {
    pub gene_type: PyGeneType,
    pub codec: Py<PyAny>,
    pub problem_builder: Py<PyAny>,
    pub inputs: Vec<PyEngineInput>,
}

#[pymethods]
impl PyEngineBuilderTwo {
    #[new]
    pub fn new(
        gene_type: String,
        codec: Py<PyAny>,
        problem_builder: Py<PyAny>,
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
        PyEngineBuilderTwo {
            gene_type,
            codec,
            problem_builder,
            inputs,
        }
    }

    pub fn build<'py>(&self, py: Python<'py>) -> PyResult<()> {
        let mut inner = EngineBuilderInner::Empty;
        let problem = self
            .problem_builder
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
                        inner = EngineBuilderInner::Float(builder);
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
                        inner = EngineBuilderInner::Int(builder);
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

        Ok(())
    }
}

impl PyEngineBuilderTwo {

    fn process_max_species_age(
        &self,
        builder: EngineBuilderInner,
        inputs: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
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
        builder: EngineBuilderInner,
        inputs: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
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
        builder: EngineBuilderInner,
        inputs: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
        self.process_single_value(builder, inputs, |builder, input| {
            let fraction = input
                .args
                .get("fraction")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.5);

            apply_to_builder!(builder, offspring_fraction(fraction))
        })
    }

    fn process_single_value<F>(
        &self,
        builder: EngineBuilderInner,
        inputs: &[&PyEngineInput],
        processor: F,
    ) -> PyResult<EngineBuilderInner>
    where
        F: Fn(EngineBuilderInner, &PyEngineInput) -> PyResult<EngineBuilderInner>,
    {
        if inputs.len() != 1 {
            return Err(PyTypeError::new_err(
                "Only one input of this type can be processed at a time",
            ));
        }
        processor(builder, inputs[0])
    }

    fn process_population_size(
        &self,
        builder: EngineBuilderInner,
        inputs: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
        self.process_single_value(builder, inputs, |builder, input| {
            let size = input
                .args
                .get("size")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100);

            apply_to_builder!(builder, population_size(size))
        })
    }

    fn process_front_range(
        &self,
        builder: EngineBuilderInner,
        inputs: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
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
                EngineBuilderInner::Int(b) => {
                    Ok(EngineBuilderInner::IntMulti(b.front_size(min..max)))
                }
                EngineBuilderInner::Float(b) => {
                    Ok(EngineBuilderInner::FloatMulti(b.front_size(min..max)))
                }
                EngineBuilderInner::Char(_) => {
                    Err(PyTypeError::new_err(
                        "process_front_range not implemented for Char gene type",
                    ))
                }
                EngineBuilderInner::Bit(_) => {
                    Err(PyTypeError::new_err(
                        "process_front_range not implemented for Bit gene type",
                    ))
                }
                EngineBuilderInner::IntMulti(b) => {
                    Ok(EngineBuilderInner::IntMulti(b.front_size(min..max)))
                }
                EngineBuilderInner::FloatMulti(b) => {
                    Ok(EngineBuilderInner::FloatMulti(b.front_size(min..max)))
                }
                EngineBuilderInner::GraphRegression(_) => {
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

    fn process_selector(
        &self,
        builder: EngineBuilderInner,
        inputs: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
        self.process_single_value(builder, inputs, |builder, input| {
            let selector: Box<dyn Select<FloatChromosome> + 'static> = match builder {
                EngineBuilderInner::Int(_) => input.convert(),
                EngineBuilderInner::Float(_) => input.convert(),
                EngineBuilderInner::Char(_) => input.convert(),
                EngineBuilderInner::Bit(_) => input.convert(),
                EngineBuilderInner::IntMulti(_) => input.convert(),
                EngineBuilderInner::FloatMulti(_) => input.convert(),
                EngineBuilderInner::GraphRegression(_) => input.convert(),
                EngineBuilderInner::Empty => {
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
        builder: EngineBuilderInner,
        input: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
        match builder {
            EngineBuilderInner::Int(builder) => {
                let alters: Vec<Box<dyn Alter<IntChromosome<i32>>>> = input.convert();
                Ok(EngineBuilderInner::Int(builder.alter(alters)))
            }
            _ => Err(PyTypeError::new_err(
                "process_alterer only implemented for Int gene type",
            )),
        }
    }

    fn process_objective(
        &self,
        builder: EngineBuilderInner,
        input: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
        if input.len() != 1 {
            return Err(PyTypeError::new_err(
                "Only one objective can be processed at a time",
            ));
        }

        let input = input[0];
        match builder {
            EngineBuilderInner::Int(builder) => {
                let objective: Objective =
                    InputConverter::<IntChromosome<i32>, Objective>::convert(input);

                Ok(match objective {
                    Objective::Single(opt) => EngineBuilderInner::Int(match opt {
                        Optimize::Minimize => builder.minimizing(),
                        Optimize::Maximize => builder.maximizing(),
                    }),
                    Objective::Multi(opts) => {
                        EngineBuilderInner::IntMulti(builder.multi_objective(opts))
                    }
                })
            }
            _ => Err(PyTypeError::new_err(
                "process_objective only implemented for Int gene type",
            )),
        }
    }

    fn process_executor(
        &self,
        builder: EngineBuilderInner,
        input: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
        if input.len() != 1 {
            return Err(PyTypeError::new_err(
                "Only one executor can be processed at a time",
            ));
        }

        let input = input[0];
        match builder {
            EngineBuilderInner::Int(builder) => {
                let executor: Executor =
                    InputConverter::<IntChromosome<i32>, Executor>::convert(input);
                Ok(EngineBuilderInner::Int(builder.executor(executor)))
            }
            _ => Err(PyTypeError::new_err(
                "process_executor only implemented for Int gene type",
            )),
        }
    }

    fn process_max_phenotype_age(
        &self,
        builder: EngineBuilderInner,
        input: &[&PyEngineInput],
    ) -> PyResult<EngineBuilderInner> {
        if input.len() != 1 {
            return Err(PyTypeError::new_err(
                "Only one max phenotype age can be processed at a time",
            ));
        }

        let input = input[0];
        let max_age = input
            .args
            .get("max_age")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(usize::MAX);
        match builder {
            EngineBuilderInner::Int(builder) => {
                Ok(EngineBuilderInner::Int(builder.max_age(max_age)))
            }
            _ => Err(PyTypeError::new_err(
                "process_max_phenotype_age only implemented for Int gene type",
            )),
        }
    }
}

impl<C> InputConverter<C, Executor> for PyEngineInput
where
    C: Chromosome,
{
    fn convert(&self) -> Executor {
        if self.input_type != PyEngineInputType::Executor {
            panic!("Input type {:?} not an executor", self.input_type);
        }

        match self.component.as_str() {
            "Serial" => Executor::Serial,
            "FixedSizedWorkerPool" => {
                let num_workers = self
                    .args
                    .get("num_workers")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                Executor::FixedSizedWorkerPool(num_workers)
            }
            "WorkerPool" => Executor::WorkerPool,
            _ => panic!("Executor type {} not yet implemented", self.component),
        }
    }
}
