use crate::AnyChromosome;
use crate::any::{CrossoverExpr, ExprDispatch, MutateExpr, PyAlterExpr, PyExpr};
use pyo3::{pyclass, pymethods};
use radiate::{AlterResult, Chromosome, Crossover, Mutate};

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyAlteration {
    pub inner: PyExpr,
}

#[pymethods]
impl PyAlteration {
    #[staticmethod]
    #[pyo3(signature = (range, target, p=1.0))]
    fn uniform(range: (f32, f32), target: String, p: f32) -> Self {
        PyAlteration {
            inner: PyAlterExpr::new()
                .name(target.clone())
                .prob(p)
                .mutate(MutateExpr::Uniform(range.0..range.1))
                .build(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (mean, stddev, target, p=1.0))]
    fn gaussian(mean: f32, stddev: f32, target: String, p: f32) -> Self {
        PyAlteration {
            inner: PyAlterExpr::new()
                .name(target.clone())
                .prob(p)
                .mutate(MutateExpr::Gaussian(mean, stddev))
                .build(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (amount, target, p=1.0))]
    fn jitter(amount: f32, target: String, p: f32) -> Self {
        PyAlteration {
            inner: PyAlterExpr::new()
                .name(target.clone())
                .prob(p)
                .mutate(MutateExpr::Jitter(amount))
                .build(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn swap(target: String, p: f32) -> Self {
        PyAlteration {
            inner: PyAlterExpr::new()
                .name(target.clone())
                .prob(p)
                .cross(CrossoverExpr::Swap)
                .build(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn mean(target: String, p: f32) -> Self {
        PyAlteration {
            inner: PyAlterExpr::new()
                .name(target.clone())
                .prob(p)
                .cross(CrossoverExpr::Mean)
                .build(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn one_point(target: String, p: f32) -> Self {
        PyAlteration {
            inner: PyAlterExpr::new()
                .name(target.clone())
                .prob(p)
                .cross(CrossoverExpr::OnePoint)
                .build(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn two_point(target: String, p: f32) -> Self {
        PyAlteration {
            inner: PyAlterExpr::new()
                .name(target.clone())
                .prob(p)
                .cross(CrossoverExpr::TwoPoint)
                .build(),
        }
    }
}

pub struct ExprMutator {
    pub alters: Vec<PyExpr>,
}

impl ExprMutator {
    pub fn new(alters: Vec<PyExpr>) -> Self {
        ExprMutator { alters }
    }
}

impl Mutate<AnyChromosome<'_>> for ExprMutator {
    fn name(&self) -> String {
        "ExpressionMutator".into()
    }

    fn mutate_chromosome(&self, chromosome: &mut AnyChromosome<'_>, _: f32) -> AlterResult {
        let mut count = 0;

        for alteration in &self.alters {
            count += chromosome.dispatch(&alteration);
        }

        count.into()
    }
}

pub struct ExprCrossover {
    pub alters: Vec<PyExpr>,
}

impl ExprCrossover {
    pub fn new(alters: Vec<PyExpr>) -> Self {
        ExprCrossover { alters }
    }
}

impl Crossover<AnyChromosome<'static>> for ExprCrossover {
    fn name(&self) -> String {
        "ExpressionCrossover".into()
    }

    fn cross_chromosomes(
        &self,
        chrom_one: &mut AnyChromosome<'static>,
        chrom_two: &mut AnyChromosome<'static>,
        _: f32,
    ) -> AlterResult {
        let mut count = 0;

        let n = std::cmp::min(chrom_one.len(), chrom_two.len());

        if n == 0 {
            return count.into();
        }

        for alter in &self.alters {
            count += (&mut *chrom_one, &mut *chrom_two).dispatch(&alter);
        }

        count.into()
    }
}
