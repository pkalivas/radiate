use core::fmt;
use radiate_core::{
    chromosomes::gene::{HasNumericSlot, apply_numeric_slot_mut},
    random_provider,
};
use std::{ops::Range, sync::Arc};

#[derive(Clone)]
pub struct MutFn<G> {
    pub(crate) f: Arc<dyn Fn(&mut G) -> usize + Send + Sync + 'static>,
    name: Option<&'static str>,
}

impl<G> MutFn<G> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut G) -> usize + Send + Sync + 'static,
    {
        Self {
            f: Arc::new(f),
            name: None,
        }
    }

    pub fn named<F>(name: &'static str, f: F) -> Self
    where
        F: Fn(&mut G) -> usize + Send + Sync + 'static,
    {
        Self {
            f: Arc::new(f),
            name: Some(name),
        }
    }

    pub fn name(&self) -> Option<&'static str> {
        self.name
    }

    #[inline]
    pub fn apply(&self, g: &mut G) -> usize {
        (self.f)(g)
    }
}

impl<G> fmt::Debug for MutFn<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.name {
            write!(f, "Mut({n})")
        } else {
            write!(f, "Mut(<closure>)")
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum MutateExpr {
    Invert,
}

#[derive(Clone, PartialEq)]
pub enum NumericMutateExpr {
    Uniform(Range<f32>),
    Gaussian(f32, f32),
    Jitter(f32),
    Arithmetic,
}

pub trait MutOp<G> {
    fn apply_single(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync>;
    fn apply_on_slice(self) -> Arc<dyn Fn(&mut [G]) -> usize + Send + Sync>;
}

impl<G: HasNumericSlot + 'static> Into<MutFn<G>> for NumericMutateExpr {
    fn into(self) -> MutFn<G> {
        let to_map = self.apply_single();
        MutFn::new(move |g: &mut G| to_map(g))
    }
}

impl<G: 'static> Into<MutFn<G>> for MutateExpr {
    fn into(self) -> MutFn<G> {
        let to_map = self.apply_single();
        MutFn::new(move |g: &mut G| to_map(g))
    }
}

impl<G> MutOp<G> for MutateExpr {
    fn apply_single(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync> {
        match self {
            _ => Arc::new(move |_: &mut G| 0),
        }
    }

    fn apply_on_slice(self) -> Arc<dyn Fn(&mut [G]) -> usize + Send + Sync> {
        match self {
            MutateExpr::Invert => Arc::new(move |gs: &mut [G]| {
                let mut count = 0;
                if gs.len() > 1 {
                    let start = random_provider::range(0..gs.len());
                    let end = random_provider::range(start + 1..gs.len());
                    gs[start..end].reverse();
                    count += 1;
                }

                count
            }),
        }
    }
}

impl<G: HasNumericSlot> MutOp<G> for NumericMutateExpr {
    fn apply_single(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync> {
        match self {
            NumericMutateExpr::Uniform(amount) => {
                Arc::new(move |g: &mut G| uniform_mutate(g, amount.clone()))
            }
            NumericMutateExpr::Gaussian(mean, std_dev) => Arc::new(move |g: &mut G| {
                let mu = mean as f64;
                let sd = (std_dev as f64).max(1e-12);
                gaussian_mutate(g, mu, sd)
            }),
            NumericMutateExpr::Jitter(amount) => {
                Arc::new(move |g: &mut G| jitter_mutate(g, amount))
            }
            _ => Arc::new(move |_: &mut G| 0),
        }
    }

    fn apply_on_slice(self) -> Arc<dyn Fn(&mut [G]) -> usize + Send + Sync> {
        match self {
            NumericMutateExpr::Uniform(amount) => Arc::new(move |gs: &mut [G]| {
                let mut count = 0;
                for g in gs.iter_mut() {
                    count += uniform_mutate(g, amount.clone());
                }
                count
            }),
            NumericMutateExpr::Gaussian(mean, std_dev) => Arc::new(move |gs: &mut [G]| {
                let mut count = 0;
                let mu = mean as f64;
                let sd = (std_dev as f64).max(1e-12);
                for g in gs.iter_mut() {
                    count += gaussian_mutate(g, mu, sd);
                }
                count
            }),
            NumericMutateExpr::Jitter(amount) => Arc::new(move |gs: &mut [G]| {
                let mut count = 0;
                for g in gs.iter_mut() {
                    count += jitter_mutate(g, amount);
                }
                count
            }),
            _ => Arc::new(move |_: &mut [G]| 0),
        }
    }
}

pub(crate) fn jitter_mutate<N: HasNumericSlot>(slot: &mut N, amount: f32) -> usize {
    slot.numeric_slot_mut()
        .map(|slot| {
            let delta = random_provider::gaussian(-1.0, 1.0) as f32 * amount;
            apply_numeric_slot_mut(
                slot,
                |x_f32| x_f32 + delta,
                |x_f64| x_f64 + delta as f64,
                |i, unsigned| {
                    let delta =
                        random_provider::gaussian(-1.0, 1.0).round() as i128 * amount as i128;
                    let y = i.saturating_add(delta);
                    if unsigned { y.max(0) } else { y }
                },
            );
            1
        })
        .unwrap_or(0)
}

pub(crate) fn gaussian_mutate<N: HasNumericSlot>(slot: &mut N, mean: f64, std_dev: f64) -> usize {
    let mu = mean;
    let sd = std_dev.max(1e-12);
    slot.numeric_slot_mut()
        .map(|slot| {
            let delta = random_provider::gaussian(mu, sd);
            apply_numeric_slot_mut(
                slot,
                |_| delta as f32,
                |_| delta,
                |i, unsigned| {
                    let y = i.saturating_add(delta.round() as i128);
                    if unsigned { y.max(0) } else { y }
                },
            );
            1
        })
        .unwrap_or(0)
}

pub(crate) fn uniform_mutate<N: HasNumericSlot>(slot: &mut N, amount: Range<f32>) -> usize {
    slot.numeric_slot_mut()
        .map(|slot| {
            let delta = random_provider::range(amount.clone());
            apply_numeric_slot_mut(
                slot,
                |x_f32| x_f32 + delta,
                |x_f64| x_f64 + delta as f64,
                |i, unsigned| {
                    let y = i.saturating_add(delta.round() as i128);
                    if unsigned { y.max(0) } else { y }
                },
            );
            1
        })
        .unwrap_or(0)
}
