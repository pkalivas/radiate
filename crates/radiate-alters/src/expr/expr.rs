use radiate_core::random_provider;

use crate::Pred;
use crate::expr::{CrossOp, CrossoverExpr, MutFn, Xover};
use std::fmt;
use std::{fmt::Debug, sync::Arc};

pub type E<G> = Arc<Expr<G>>;

#[derive(Clone)]
pub enum SelectExpr<G> {
    All,
    Some(Pred<G>),
}

#[derive(Clone, Default)]
pub enum Expr<G> {
    #[default]
    NoOp,
    Index(usize, E<G>),
    Mut(MutFn<G>),
    Cross(Xover<G>, E<G>, E<G>),
    Filter(Pred<G>, E<G>),
    Prob(f32, E<G>),
    Seq(Vec<Expr<G>>),
    Select(SelectExpr<G>, E<G>),
    MapEach(Arc<Expr<G>>),
}

enum Ctx<'a, G> {
    One(&'a mut G),
    Many(&'a mut [G]),
    Pair(&'a mut G, &'a mut G),
    ManyPairs(&'a mut [G], &'a mut [G]),
}

impl<'a, G> Ctx<'a, G> {
    #[inline]
    fn prob(p: f32) -> bool {
        p > 0.0 && p <= 1.0 && random_provider::random::<f32>() < p
    }
}

impl<G> Expr<G> {
    #[inline]
    pub fn apply_slice(&self, xs: &mut [G]) -> usize {
        let mut ctx = Ctx::Many(xs);
        self.apply_in(&mut ctx)
    }

    #[inline]
    pub fn apply_gene(&self, x: &mut G) -> usize {
        let mut ctx = Ctx::One(x);
        self.apply_in(&mut ctx)
    }

    #[inline]
    pub fn apply_pairs(&self, a: &mut [G], b: &mut [G]) -> usize
    where
        CrossoverExpr: CrossOp<G>,
    {
        let mut ctx = Ctx::ManyPairs(a, b);
        self.apply_in(&mut ctx)
    }

    #[inline]
    pub fn apply_pair_genes(&self, a: &mut G, b: &mut G) -> usize
    where
        CrossoverExpr: CrossOp<G>,
    {
        let mut ctx = Ctx::Pair(a, b);
        self.apply_in(&mut ctx)
    }

    #[inline(always)]
    fn apply_in(&self, ctx: &mut Ctx<'_, G>) -> usize
    where
        CrossoverExpr: CrossOp<G>,
    {
        match self {
            Expr::NoOp => 0,

            Expr::Seq(list) => list.iter().map(|e| e.apply_in(ctx)).sum(),

            Expr::Prob(p, inner) => {
                if Ctx::<G>::prob(*p) {
                    inner.apply_in(ctx)
                } else {
                    0
                }
            }

            Expr::Select(SelectExpr::All, inner) => match ctx {
                Ctx::One(x) => inner.apply_in(&mut Ctx::One(x)),
                Ctx::Many(xs) => xs
                    .iter_mut()
                    .map(|x| inner.apply_in(&mut Ctx::One(x)))
                    .sum(),
                Ctx::Pair(a, b) => inner.apply_in(&mut Ctx::Pair(a, b)),
                Ctx::ManyPairs(a, b) => inner.apply_in(&mut Ctx::ManyPairs(a, b)),
            },

            Expr::Select(SelectExpr::Some(pred), inner) => match ctx {
                Ctx::One(x) => {
                    if pred.test(x) {
                        inner.apply_in(&mut Ctx::One(x))
                    } else {
                        0
                    }
                }
                Ctx::Many(xs) => xs
                    .iter_mut()
                    .filter(|x| pred.test(x))
                    .map(|x| inner.apply_in(&mut Ctx::One(x)))
                    .sum(),
                Ctx::Pair(a, b) => {
                    if pred.test(a) {
                        inner.apply_in(&mut Ctx::Pair(*a, *b))
                    } else {
                        0
                    }
                }
                Ctx::ManyPairs(a, b) => a
                    .iter_mut()
                    .zip(b.iter_mut())
                    .filter(|(x, _)| pred.test(x))
                    .map(|(x, y)| inner.apply_in(&mut Ctx::Pair(x, y)))
                    .sum(),
            },

            Expr::Index(i, inner) => match ctx {
                Ctx::Many(xs) => {
                    if *i < xs.len() {
                        inner.apply_in(&mut Ctx::One(&mut xs[*i]))
                    } else {
                        0
                    }
                }
                Ctx::ManyPairs(a, b) => {
                    let n = a.len().min(b.len());
                    if *i < n {
                        inner.apply_in(&mut Ctx::Pair(&mut a[*i], &mut b[*i]))
                    } else {
                        0
                    }
                }
                // On One/Pair, Index is a no-op (or route to inner)
                Ctx::One(x) => inner.apply_in(&mut Ctx::One(x)),
                Ctx::Pair(a, b) => inner.apply_in(&mut Ctx::Pair(*a, *b)),
            },

            Expr::Filter(pred, inner) => match ctx {
                Ctx::One(x) => {
                    if pred.test(x) {
                        inner.apply_in(&mut Ctx::One(x))
                    } else {
                        0
                    }
                }
                Ctx::Many(xs) => xs
                    .iter_mut()
                    .filter(|x| pred.test(x))
                    .map(|x| inner.apply_in(&mut Ctx::One(x)))
                    .sum(),
                Ctx::Pair(a, b) => {
                    if pred.test(a) {
                        inner.apply_in(&mut Ctx::Pair(*a, *b))
                    } else {
                        0
                    }
                }
                Ctx::ManyPairs(a, b) => a
                    .iter_mut()
                    .zip(b.iter_mut())
                    .filter(|(x, _)| pred.test(x)) // || pred.test(y))
                    .map(|(x, y)| inner.apply_in(&mut Ctx::Pair(x, y)))
                    .sum(),
            },

            Expr::MapEach(inner) => match ctx {
                Ctx::One(x) => inner.apply_in(&mut Ctx::One(x)),
                Ctx::Pair(a, b) => inner.apply_in(&mut Ctx::Pair(*a, *b)),
                Ctx::Many(xs) => xs
                    .iter_mut()
                    .map(|x| inner.apply_in(&mut Ctx::One(x)))
                    .sum(),
                Ctx::ManyPairs(a, b) => a
                    .iter_mut()
                    .zip(b.iter_mut())
                    .map(|(x, y)| inner.apply_in(&mut Ctx::Pair(x, y)))
                    .sum(),
            },

            Expr::Mut(f) => match ctx {
                Ctx::One(x) => f.apply(*x),
                Ctx::Many(xs) => xs.iter_mut().map(|x| f.apply(x)).sum(),
                Ctx::Pair(a, b) => f.apply(*a) + f.apply(*b),
                Ctx::ManyPairs(a, b) => a
                    .iter_mut()
                    .zip(b.iter_mut())
                    .map(|(x, y)| f.apply(x) + f.apply(y))
                    .sum(),
            },

            Expr::Cross(kind, lhs, rhs) => match ctx {
                Ctx::Pair(a, b) => {
                    let mut changed = 0;
                    changed += lhs.apply_in(&mut Ctx::One(*a));
                    changed += rhs.apply_in(&mut Ctx::One(*b));
                    changed + kind.op().apply_on_pair(*a, *b)
                }
                Ctx::ManyPairs(a, b) => {
                    let mut changed = 0;
                    changed += lhs.apply_in(&mut Ctx::Many(*a));
                    changed += rhs.apply_in(&mut Ctx::Many(*b));
                    changed + kind.op().apply_on_slices(*a, *b)
                }
                // Crossing doesn’t make sense on One/Many; treat as no-op.
                _ => 0,
            },
        }
    }
}

#[allow(dead_code)]
pub mod dsl {

    use super::*;
    use crate::expr::{MutateExpr, NumericCrossoverExpr, NumericMutateExpr};
    use radiate_core::{BoundedGene, Gene, chromosomes::gene::HasNumericSlot};
    use std::ops::Range;

    pub fn all<G>(inner: Expr<G>) -> Expr<G> {
        Expr::Select(SelectExpr::All, Arc::new(inner))
    }

    pub fn index<G>(i: usize, inner: Expr<G>) -> Expr<G> {
        Expr::Index(i, Arc::new(inner))
    }

    pub fn filter<G>(pred: impl Fn(&G) -> bool + Send + Sync + 'static, inner: Expr<G>) -> Expr<G> {
        Expr::Filter(Pred::new(pred), Arc::new(inner))
    }

    pub fn prob<G>(p: f32, inner: Expr<G>) -> Expr<G> {
        Expr::Prob(p, Arc::new(inner))
    }

    pub fn seq<G>(xs: impl Into<Vec<Expr<G>>>) -> Expr<G> {
        Expr::Seq(xs.into())
    }

    pub fn map_each<G>(inner: Expr<G>) -> Expr<G> {
        Expr::MapEach(Arc::new(inner))
    }

    pub fn prob_each<G>(p: f32, inner: Expr<G>) -> Expr<G> {
        map_each(Expr::Prob(p, Arc::new(inner)))
    }

    pub fn mutate<G, M: Into<MutFn<G>>>(m: M) -> Expr<G> {
        Expr::Mut(m.into())
    }

    pub fn cross<G>(
        cross: impl Into<Xover<G>>,
        lhs: Option<Expr<G>>,
        rhs: Option<Expr<G>>,
    ) -> Expr<G> {
        Expr::Cross(
            cross.into(),
            Arc::new(lhs.unwrap_or(Expr::NoOp)),
            Arc::new(rhs.unwrap_or(Expr::NoOp)),
        )
    }

    /// Crossover operations.
    pub fn one_point_cross<G>() -> Expr<G> {
        cross(CrossoverExpr::OnePoint, None, None)
    }

    pub fn two_point_cross<G>() -> Expr<G> {
        cross(CrossoverExpr::TwoPoint, None, None)
    }

    pub fn blend_cross<G: HasNumericSlot + 'static>(alpha: f32) -> Expr<G> {
        cross(NumericCrossoverExpr::Blend(alpha), None, None)
    }

    pub fn intermediate_cross<G: HasNumericSlot + 'static>(alpha: f32) -> Expr<G> {
        cross(NumericCrossoverExpr::Intermediate(alpha), None, None)
    }

    pub fn mean_cross<G: HasNumericSlot + 'static>() -> Expr<G> {
        cross(NumericCrossoverExpr::Mean, None, None)
    }

    pub fn swap_cross<G>() -> Expr<G> {
        cross(CrossoverExpr::Swap, None, None)
    }

    /// Mutation operations
    pub fn uniform_mutate<G: HasNumericSlot + 'static>(range: Range<f32>) -> Expr<G> {
        mutate(NumericMutateExpr::Uniform(range))
    }

    pub fn gausian_mutate<G: HasNumericSlot + 'static>(mean: f32, std_dev: f32) -> Expr<G> {
        mutate(NumericMutateExpr::Gaussian(mean, std_dev))
    }

    pub fn jitter_mutate<G: HasNumericSlot + 'static>(amount: f32) -> Expr<G> {
        mutate(NumericMutateExpr::Jitter(amount))
    }

    pub fn inversion_mutate<G: 'static>() -> Expr<G> {
        mutate(MutateExpr::Invert)
    }

    pub fn gaussian_by_bounds_generic<G>() -> Expr<G>
    where
        G: HasNumericSlot + BoundedGene + Gene,
        G::Allele: Copy + Into<f64>,
    {
        mutate(MutFn::named("gauss-by-bounds", |g: &mut G| {
            let min = (*g.min()).into() as f64;
            let max = (*g.max()).into() as f64;
            let mean = (*g.allele()).into() as f64;
            let std = (max - min) * 0.25_f64;
            super::super::mutate::gaussian_mutate(g, mean, std)
        }))
    }
}

impl<G> Debug for Expr<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.dump_tree())
    }
}

#[cfg(test)]
mod tests {
    use radiate_core::{Chromosome, FloatChromosome, FloatGene, Gene};

    use crate::expr::NumericMutateExpr;
    use crate::expr::builder::genes;

    use super::dsl::*;
    use super::*;

    #[test]
    fn mutate_float_chromosome() {
        // build a chromosome
        let mut chromo = FloatChromosome::from(vec![0.0, 1.0, 2.0, 3.0]);

        let expr: Expr<FloatGene> = prob(1.0, all(uniform_mutate(-1.0..1.0)));
        let changed = expr.apply_slice(chromo.genes_mut());

        assert!(changed > 0);
    }

    #[test]
    fn filter_then_mutate_index() {
        let mut chromo = FloatChromosome::from(vec![0.0, 1.0, 2.0, 3.0]);

        // Only mutate genes whose allele < 2.0, but only at index 1
        let expr: Expr<FloatGene> = index(
            1,
            filter(
                |g: &FloatGene| *g.allele() < 2.0,
                mutate(NumericMutateExpr::Uniform(-0.5..0.5)),
            ),
        );

        let before = chromo.genes()[1].clone();
        let changed = expr.apply_slice(chromo.genes_mut());

        println!("Before: {:?}", chromo);

        assert!(changed == 1);
        assert_ne!(*before.allele(), *chromo.genes()[1].allele());
    }

    #[test]
    fn seq_pipeline() {
        let mut chromo = FloatChromosome::from(vec![0.0, 1.0, 2.0, 3.0]);

        // Pipeline: jitter all, then (prob 0.5) jitter index 0 again
        let _: Expr<FloatGene> = Expr::Seq(vec![
            all(mutate(NumericMutateExpr::Uniform(-0.1..0.1))),
            prob(0.5, index(0, uniform_mutate(-0.3..0.3))),
        ]);

        let expr: Expr<FloatGene> = seq([
            all(mutate(MutFn::named("jitter±0.1", |g: &mut FloatGene| {
                *g.allele_mut() += radiate_core::random_provider::range(-0.1..0.1);
                1
            }))),
            prob(
                0.5,
                index(
                    0,
                    mutate(MutFn::named("kick0", |g: &mut FloatGene| {
                        *g.allele_mut() += 0.3;
                        1
                    })),
                ),
            ),
        ]);

        // let cross_expr = cross::<FloatGene>(
        //     Xover::named("one-point", OnePointXover::new(0.7)),
        //     all(mutate(MutFn::named("small-noise", |g| {
        //         *g.allele_mut() *= 1.01;
        //         1
        //     }))),
        //     all(mutate(MutFn::named("small-noise", |g| {
        //         *g.allele_mut() *= 0.99;
        //         1
        //     }))),
        // );

        println!("Tree: {:?}", expr.dump_tree());

        let _ = expr.apply_slice(chromo.genes_mut());
    }

    #[test]
    fn one_point_crossover_float() {
        let mut a = FloatChromosome::from(vec![0.0; 10]);
        let mut b = FloatChromosome::from(vec![1.0; 10]);

        // Cross whole chromosome with a one-point crossover
        let _: Expr<FloatGene> = genes()
            .two_point_cross()
            .map_each(prob(0.5, mean_cross()))
            .build();

        let expr = genes()
            .all()
            .map_each(prob(
                0.3, // 30% of loci
                cross(
                    CrossoverExpr::TwoPoint,
                    Some(jitter_mutate(0.05)), // only parent A
                    Some(jitter_mutate(0.05)), // only parent B
                ),
            ))
            .build();

        let changed = expr.apply_pairs(a.genes_mut(), b.genes_mut());
        println!(
            "Changed: {:?}",
            a.iter().map(|g| g.allele()).collect::<Vec<_>>()
        );
        println!(
            "Changed: {:?}",
            b.iter().map(|g| g.allele()).collect::<Vec<_>>()
        );

        println!("Tree: {:?}", expr.dump_tree());
        assert!(changed > 0);
    }
}
