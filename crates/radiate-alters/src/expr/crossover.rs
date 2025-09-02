use core::fmt;
use std::sync::Arc;

use radiate_core::{chromosomes::gene::HasNumericSlot, random_provider};

#[derive(Clone)]
pub struct Xover<G> {
    op: Arc<dyn CrossOp<G> + Send + Sync + 'static>,
    name: Option<&'static str>,
}

impl<G> Xover<G> {
    pub fn new(op: impl CrossOp<G> + Send + Sync + 'static) -> Self {
        Self {
            op: Arc::new(op),
            name: None,
        }
    }

    pub fn named(name: &'static str, op: impl CrossOp<G> + Send + Sync + 'static) -> Self {
        Self {
            op: Arc::new(op),
            name: Some(name),
        }
    }

    pub fn name(&self) -> Option<&'static str> {
        self.name
    }

    #[inline]
    pub fn op(&self) -> &dyn CrossOp<G> {
        &*self.op
    }
}

impl<G> fmt::Debug for Xover<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.name {
            write!(f, "Xover({n})")
        } else {
            write!(f, "Xover(<op>)")
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum CrossoverExpr {
    OnePoint,
    TwoPoint,
    Swap,
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum NumericCrossoverExpr {
    Blend(f32),
    Mean,
}

pub trait CrossOp<G> {
    fn apply_on_slices(&self, a: &mut [G], b: &mut [G]) -> usize;

    fn apply_on_pair(&self, _a: &mut G, _b: &mut G) -> usize;
}

impl<G: HasNumericSlot> CrossOp<G> for NumericCrossoverExpr {
    fn apply_on_slices(&self, a: &mut [G], b: &mut [G]) -> usize {
        match self {
            NumericCrossoverExpr::Blend(factor) => {
                let mut count = 0;
                for (a_gene, b_gene) in a.iter_mut().zip(b.iter_mut()) {
                    count += crate::crossovers::blend::blend_crossover(a_gene, b_gene, *factor);
                }
                count
            }
            NumericCrossoverExpr::Mean => {
                let mut count = 0;
                for (a_gene, b_gene) in a.iter_mut().zip(b.iter_mut()) {
                    count += crate::crossovers::mean::crossover_mean(a_gene, b_gene);
                }
                count
            }
        }
    }

    fn apply_on_pair(&self, a: &mut G, b: &mut G) -> usize {
        match self {
            NumericCrossoverExpr::Blend(factor) => {
                crate::crossovers::blend::blend_crossover(a, b, *factor)
            }
            NumericCrossoverExpr::Mean => crate::crossovers::mean::crossover_mean(a, b),
        }
    }
}

impl<G> CrossOp<G> for CrossoverExpr {
    fn apply_on_slices(&self, a: &mut [G], b: &mut [G]) -> usize {
        match self {
            CrossoverExpr::OnePoint => crate::crossovers::multipoint::crossover_single_point(a, b),
            CrossoverExpr::TwoPoint => {
                crate::crossovers::multipoint::crossover_multi_point(a, b, 2)
            }
            CrossoverExpr::Swap => {
                let n = a.len().min(b.len());

                for i in 0..n {
                    if random_provider::bool(0.5) {
                        std::mem::swap(&mut a[i], &mut b[i]);
                    }
                }

                n
            }
        }
    }

    fn apply_on_pair(&self, a: &mut G, b: &mut G) -> usize {
        match self {
            CrossoverExpr::Swap => {
                std::mem::swap(a, b);
                1
            }
            _ => 0,
        }
    }
}
