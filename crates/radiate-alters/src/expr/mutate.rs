// use core::fmt;
// use radiate_core::{
//     chromosomes::gene::{HasNumericSlot, apply_numeric_slot_mut},
//     random_provider,
// };
// use std::{ops::Range, sync::Arc};

// #[derive(Clone)]
// pub struct MutFn<G> {
//     pub(crate) f: Arc<dyn Fn(&mut G) -> usize + Send + Sync + 'static>,
//     name: Option<&'static str>,
// }

// impl<G> MutFn<G> {
//     pub fn new<F>(f: F) -> Self
//     where
//         F: Fn(&mut G) -> usize + Send + Sync + 'static,
//     {
//         Self {
//             f: Arc::new(f),
//             name: None,
//         }
//     }

//     pub fn named<F>(name: &'static str, f: F) -> Self
//     where
//         F: Fn(&mut G) -> usize + Send + Sync + 'static,
//     {
//         Self {
//             f: Arc::new(f),
//             name: Some(name),
//         }
//     }

//     pub fn name(&self) -> Option<&'static str> {
//         self.name
//     }

//     #[inline]
//     pub fn apply(&self, g: &mut G) -> usize {
//         (self.f)(g)
//     }
// }

// impl<G> fmt::Debug for MutFn<G> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         if let Some(n) = self.name {
//             write!(f, "Mut({n})")
//         } else {
//             write!(f, "Mut(<closure>)")
//         }
//     }
// }

// #[derive(Clone, PartialEq)]
// pub enum MutateExpr {
//     Invert,
// }

// #[derive(Clone, PartialEq)]
// pub enum NumericMutateExpr {
//     Uniform(Range<f32>),
//     Gaussian(f32, f32),
//     Jitter(f32),
//     Arithmetic,
// }

// pub trait MutOp<G> {
//     fn apply_single(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync>;
//     fn apply_on_slice(self) -> Arc<dyn Fn(&mut [G]) -> usize + Send + Sync>;
// }

// impl<G: HasNumericSlot + 'static> Into<MutFn<G>> for NumericMutateExpr {
//     fn into(self) -> MutFn<G> {
//         let to_map = self.apply_single();
//         MutFn::new(move |g: &mut G| to_map(g))
//     }
// }

// impl<G: 'static> Into<MutFn<G>> for MutateExpr {
//     fn into(self) -> MutFn<G> {
//         let to_map = self.apply_single();
//         MutFn::new(move |g: &mut G| to_map(g))
//     }
// }

// impl<G> MutOp<G> for MutateExpr {
//     fn apply_single(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync> {
//         match self {
//             _ => Arc::new(move |_: &mut G| 0),
//         }
//     }

//     fn apply_on_slice(self) -> Arc<dyn Fn(&mut [G]) -> usize + Send + Sync> {
//         match self {
//             MutateExpr::Invert => Arc::new(move |gs: &mut [G]| {
//                 let mut count = 0;
//                 if gs.len() > 1 {
//                     let start = random_provider::range(0..gs.len());
//                     let end = random_provider::range(start + 1..gs.len());
//                     gs[start..end].reverse();
//                     count += 1;
//                 }

//                 count
//             }),
//         }
//     }
// }

// impl<G: HasNumericSlot> MutOp<G> for NumericMutateExpr {
//     fn apply_single(self) -> Arc<dyn Fn(&mut G) -> usize + Send + Sync> {
//         match self {
//             NumericMutateExpr::Uniform(amount) => {
//                 Arc::new(move |g: &mut G| uniform_mutate(g, amount.clone()))
//             }
//             NumericMutateExpr::Gaussian(mean, std_dev) => Arc::new(move |g: &mut G| {
//                 let mu = mean as f64;
//                 let sd = (std_dev as f64).max(1e-12);
//                 gaussian_mutate(g, mu, sd)
//             }),
//             NumericMutateExpr::Jitter(amount) => {
//                 Arc::new(move |g: &mut G| jitter_mutate(g, amount))
//             }
//             _ => Arc::new(move |_: &mut G| 0),
//         }
//     }

//     fn apply_on_slice(self) -> Arc<dyn Fn(&mut [G]) -> usize + Send + Sync> {
//         match self {
//             NumericMutateExpr::Uniform(amount) => Arc::new(move |gs: &mut [G]| {
//                 let mut count = 0;
//                 for g in gs.iter_mut() {
//                     count += uniform_mutate(g, amount.clone());
//                 }
//                 count
//             }),
//             NumericMutateExpr::Gaussian(mean, std_dev) => Arc::new(move |gs: &mut [G]| {
//                 let mut count = 0;
//                 let mu = mean as f64;
//                 let sd = (std_dev as f64).max(1e-12);
//                 for g in gs.iter_mut() {
//                     count += gaussian_mutate(g, mu, sd);
//                 }
//                 count
//             }),
//             NumericMutateExpr::Jitter(amount) => Arc::new(move |gs: &mut [G]| {
//                 let mut count = 0;
//                 for g in gs.iter_mut() {
//                     count += jitter_mutate(g, amount);
//                 }
//                 count
//             }),
//             _ => Arc::new(move |_: &mut [G]| 0),
//         }
//     }
// }

// pub(crate) fn jitter_mutate<N: HasNumericSlot>(slot: &mut N, amount: f32) -> usize {
//     slot.numeric_slot_mut()
//         .map(|slot| {
//             let delta = random_provider::gaussian(-1.0, 1.0) as f32 * amount;
//             apply_numeric_slot_mut(
//                 slot,
//                 |x_f32| x_f32 + delta,
//                 |x_f64| x_f64 + delta as f64,
//                 |i, unsigned| {
//                     let delta =
//                         random_provider::gaussian(-1.0, 1.0).round() as i128 * amount as i128;
//                     let y = i.saturating_add(delta);
//                     if unsigned { y.max(0) } else { y }
//                 },
//             );
//             1
//         })
//         .unwrap_or(0)
// }

// pub(crate) fn gaussian_mutate<N: HasNumericSlot>(slot: &mut N, mean: f64, std_dev: f64) -> usize {
//     let mu = mean;
//     let sd = std_dev.max(1e-12);
//     slot.numeric_slot_mut()
//         .map(|slot| {
//             let delta = random_provider::gaussian(mu, sd);
//             apply_numeric_slot_mut(
//                 slot,
//                 |_| delta as f32,
//                 |_| delta,
//                 |i, unsigned| {
//                     let y = i.saturating_add(delta.round() as i128);
//                     if unsigned { y.max(0) } else { y }
//                 },
//             );
//             1
//         })
//         .unwrap_or(0)
// }

// pub(crate) fn uniform_mutate<N: HasNumericSlot>(slot: &mut N, amount: Range<f32>) -> usize {
//     slot.numeric_slot_mut()
//         .map(|slot| {
//             let delta = random_provider::range(amount.clone());
//             apply_numeric_slot_mut(
//                 slot,
//                 |x_f32| x_f32 + delta,
//                 |x_f64| x_f64 + delta as f64,
//                 |i, unsigned| {
//                     let y = i.saturating_add(delta.round() as i128);
//                     if unsigned { y.max(0) } else { y }
//                 },
//             );
//             1
//         })
//         .unwrap_or(0)
// }

use core::fmt;
use radiate_core::{
    ArithmeticGene, BoundedGene, Gene,
    chromosomes::gene::{
        HasNumericSlot, NumericAllele, NumericGene, apply_numeric_slot_mut, slot_add_scalar,
        slot_set_scalar,
    },
    random_provider,
};
use std::ops::{Mul, Range};

#[derive(Clone)]
pub struct DataFn<G, D> {
    f: fn(&mut G, &D) -> usize,
    d: D,
}

impl<G, D> DataFn<G, D> {
    #[inline]
    fn call(&self, g: &mut G) -> usize {
        (self.f)(g, &self.d)
    }
}

#[derive(Clone)]
pub enum MutKernel<G> {
    Fn(fn(&mut G) -> usize),

    Uniform(DataFn<G, Range<f32>>),
    Gaussian(DataFn<G, (f64, f64)>),
    Jitter(DataFn<G, f32>),

    Invert,
}

#[derive(Clone)]
pub struct MutFn<G> {
    k: MutKernel<G>,
    name: Option<&'static str>,
}

impl<G> MutFn<G> {
    #[inline]
    pub fn from_fn_ptr(f: fn(&mut G) -> usize) -> Self {
        Self {
            k: MutKernel::Fn(f),
            name: None,
        }
    }
    #[inline]
    pub fn named_fn(name: &'static str, f: fn(&mut G) -> usize) -> Self {
        Self {
            k: MutKernel::Fn(f),
            name: Some(name),
        }
    }

    #[inline]
    pub fn kernel(k: MutKernel<G>) -> Self {
        Self { k, name: None }
    }

    #[inline]
    pub fn named_kernel(name: &'static str, k: MutKernel<G>) -> Self {
        Self {
            k,
            name: Some(name),
        }
    }

    #[inline]
    pub fn name(&self) -> Option<&'static str> {
        self.name
    }

    #[inline]
    pub fn apply(&self, g: &mut G) -> usize {
        match &self.k {
            MutKernel::Fn(fptr) => fptr(g),
            MutKernel::Uniform(df) => df.call(g),
            MutKernel::Gaussian(df) => df.call(g),
            MutKernel::Jitter(df) => df.call(g),
            MutKernel::Invert => 0,
        }
    }

    #[inline]
    pub fn apply_slice(&self, xs: &mut [G]) -> usize {
        match &self.k {
            MutKernel::Invert => invert_random_segment(xs),
            _ => {
                let mut acc = 0;
                for x in xs {
                    acc += self.apply(x);
                }
                acc
            }
        }
    }
}

impl<G> fmt::Debug for MutFn<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.name {
            write!(f, "Mut({n})")
        } else {
            write!(f, "Mut(<kernel>)")
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

impl<G> From<NumericMutateExpr> for MutFn<G>
where
    G: NumericGene,
    G::Allele: NumericAllele,
{
    #[inline]
    fn from(m: NumericMutateExpr) -> Self {
        match m {
            NumericMutateExpr::Uniform(r) => MutFn::kernel(MutKernel::Uniform(DataFn {
                f: apply_uniform::<G>,
                d: r,
            })),
            NumericMutateExpr::Gaussian(mean, std) => {
                MutFn::kernel(MutKernel::Fn(apply_gaussian::<G>))
            }
            NumericMutateExpr::Jitter(a) => MutFn::kernel(MutKernel::Jitter(DataFn {
                f: apply_jitter::<G>,
                d: a,
            })),
            NumericMutateExpr::Arithmetic => MutFn::kernel(MutKernel::Fn(|_: &mut G| 0)),
        }
    }
}

impl<G> From<MutateExpr> for MutFn<G> {
    #[inline]
    fn from(m: MutateExpr) -> Self {
        match m {
            MutateExpr::Invert => MutFn::kernel(MutKernel::Invert),
        }
    }
}

// #[inline]
// fn apply_uniform<G: HasNumericSlot>(g: &mut G, r: &Range<f32>) -> usize {
//     let delta = random_provider::range(r.clone());
//     g.numeric_slot_mut()
//         .map(|slot| {
//             apply_numeric_slot_mut(
//                 slot,
//                 |x_f32| x_f32 + delta,
//                 |x_f64| x_f64 + delta as f64,
//                 |i, unsigned| {
//                     let y = i.saturating_add(delta.round() as i128);
//                     if unsigned { y.max(0) } else { y }
//                 },
//             );
//             1
//         })
//         .unwrap_or(0)
// }

// #[inline]
// pub(crate) fn apply_gaussian<G: HasNumericSlot>(g: &mut G, p: &(f64, f64)) -> usize {
//     let mu = p.0;
//     let sd = p.1.max(1e-12);
//     let delta = random_provider::gaussian(mu, sd);

//     g.numeric_slot_mut()
//         .map(|slot| {
//             apply_numeric_slot_mut(
//                 slot,
//                 |_| delta as f32,
//                 |_| delta,
//                 |i, unsigned| {
//                     let y = i.saturating_add(delta.round() as i128);
//                     if unsigned { y.max(0) } else { y }
//                 },
//             );
//             1
//         })
//         .unwrap_or(0)
// }

// #[inline]
// fn apply_jitter<G: HasNumericSlot>(g: &mut G, amount: &f32) -> usize {
//     g.numeric_slot_mut()
//         .map(|slot| {
//             let delta = random_provider::gaussian(-1.0, 1.0) as f32 * amount;
//             apply_numeric_slot_mut(
//                 slot,
//                 |x_f32| x_f32 + delta,
//                 |x_f64| x_f64 + delta as f64,
//                 |i, unsigned| {
//                     let delta =
//                         random_provider::gaussian(-1.0, 1.0).round() as i128 * *amount as i128;
//                     let y = i.saturating_add(delta);
//                     if unsigned { y.max(0) } else { y }
//                 },
//             );
//             1
//         })
//         .unwrap_or(0)
// }

#[inline]
fn invert_random_segment<G>(xs: &mut [G]) -> usize {
    if xs.len() > 1 {
        let start = random_provider::range(0..xs.len() - 1);
        let end = random_provider::range(start + 1..xs.len());
        xs[start..end].reverse();
        1
    } else {
        0
    }
}

#[inline(always)]
fn apply_uniform<G>(g: &mut G, r: &Range<f32>) -> usize
where
    G: NumericGene,
    G::Allele: NumericAllele,
{
    if let Some(slot) = g.numeric_slot_mut() {
        let delta = random_provider::range(r.clone());
        slot_add_scalar(slot, delta);
        1
    } else {
        0
    }
}

#[inline(always)]
pub(crate) fn apply_gaussian<G>(g: &mut G) -> usize
where
    G: NumericGene,
    G::Allele: NumericAllele,
{
    let min = (*g.min()).as_f64();
    let max = (*g.max()).as_f64();

    let std_dev = (max - min) * 0.25;
    let value = (g.allele()).as_f64();

    let val = random_provider::gaussian(value, std_dev);
    g.allele_mut().set_f64(val);
    // *g.allele_mut() = val;
    // if let Some(slot) = g.numeric_slot_mut() {
    //     slot_set_scalar(slot, val);
    //     1
    // } else {
    //     0
    // }

    1
}

#[inline(always)]
fn apply_jitter<G>(g: &mut G, amount: &f32) -> usize
where
    G: NumericGene,
    G::Allele: NumericAllele,
{
    if let Some(slot) = g.numeric_slot_mut() {
        let z = random_provider::gaussian(-1.0, 1.0) as f32; // one RNG draw
        slot_add_scalar(slot, z * *amount);
        1
    } else {
        0
    }
}
