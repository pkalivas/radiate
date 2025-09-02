use core::fmt;
use radiate_core::{
    chromosomes::gene::{HasNumericSlot, apply_numeric_slot_mut},
    random_provider,
};
use std::{ops::Range, sync::Arc};

#[derive(Clone)]
pub struct MutFn<G> {
    f: Arc<dyn Fn(&mut G) -> usize + Send + Sync + 'static>,
    name: Option<&'static str>,
}

impl<G> MutFn<G> {
    pub fn new(f: impl Fn(&mut G) -> usize + Send + Sync + 'static) -> Self {
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

impl<G> IntoMut<G> for MutFn<G> {
    fn into_mut(self) -> MutFn<G> {
        self
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
#[allow(dead_code)]
pub enum MutateExpr {
    Invert,
}

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub enum NumericMutateExpr {
    Uniform(Range<f32>),
    Gaussian(f32, f32),
    Jitter(f32),
    Arithmetic,
}

pub trait IntoMut<G> {
    fn into_mut(self) -> MutFn<G>;
}

pub trait MutOp<G> {
    fn to_map(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync>;
}

impl<G: HasNumericSlot + 'static> IntoMut<G> for NumericMutateExpr {
    fn into_mut(self) -> MutFn<G> {
        let to_map = self.to_map();
        MutFn::new(move |g: &mut G| to_map(g))
    }
}

impl<G> MutOp<G> for MutateExpr {
    fn to_map(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync> {
        match self {
            _ => Arc::new(move |_: &mut G| 0),
        }
    }
}

impl<G: HasNumericSlot> MutOp<G> for NumericMutateExpr {
    fn to_map(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync> {
        match self {
            NumericMutateExpr::Uniform(amount) => Arc::new(move |g: &mut G| {
                g.numeric_slot_mut()
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
            }),
            NumericMutateExpr::Gaussian(mean, std_dev) => Arc::new(move |g: &mut G| {
                let mu = mean as f64;
                let sd = (std_dev as f64).max(1e-12);
                g.numeric_slot_mut()
                    .map(|slot| {
                        apply_numeric_slot_mut(
                            slot,
                            |x_f32| {
                                let delta = random_provider::gaussian(mu, sd) as f32;
                                x_f32 + delta
                            },
                            |x_f64| {
                                let delta = random_provider::gaussian(mu, sd);
                                x_f64 + delta
                            },
                            |i, unsigned| {
                                // Integer: gaussian delta rounded to nearest int
                                let delta = random_provider::gaussian(mu, sd).round() as i128;
                                let y = i.saturating_add(delta);
                                if unsigned { y.max(0) } else { y }
                            },
                        );

                        1
                    })
                    .unwrap_or(0)
            }),
            NumericMutateExpr::Jitter(amount) => Arc::new(move |g: &mut G| {
                g.numeric_slot_mut()
                    .map(|slot| {
                        apply_numeric_slot_mut(
                            slot,
                            |x_f32| {
                                let delta = random_provider::gaussian(-1.0, 1.0) as f32 * amount;
                                x_f32 + delta
                            },
                            |x_f64| {
                                let delta = random_provider::gaussian(-1.0, 1.0) * amount as f64;
                                x_f64 + delta
                            },
                            |i, unsigned| {
                                // Integer: gaussian delta rounded to nearest int
                                let delta = random_provider::gaussian(-1.0, 1.0).round() as i128
                                    * amount as i128;
                                let y = i.saturating_add(delta);
                                if unsigned { y.max(0) } else { y }
                            },
                        );

                        1
                    })
                    .unwrap_or(0)
            }),
            _ => Arc::new(move |_: &mut G| 0),
        }
    }
}
