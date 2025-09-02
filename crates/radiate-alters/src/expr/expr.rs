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
        p > 0.0 && p <= 1.0 && radiate_core::random_provider::random::<f32>() < p
    }
}

#[allow(dead_code)]
impl<G> Expr<G> {
    pub fn apply_slice(&self, xs: &mut [G]) -> usize {
        let mut ctx = Ctx::Many(xs);
        self.apply_in(&mut ctx)
    }

    pub fn apply_gene(&self, x: &mut G) -> usize {
        let mut ctx = Ctx::One(x);
        self.apply_in(&mut ctx)
    }

    pub fn apply_pairs(&self, a: &mut [G], b: &mut [G]) -> usize
    where
        CrossoverExpr: CrossOp<G>,
    {
        let mut ctx = Ctx::ManyPairs(a, b);
        self.apply_in(&mut ctx)
    }

    pub fn apply_pair_genes(&self, a: &mut G, b: &mut G) -> usize
    where
        CrossoverExpr: CrossOp<G>,
    {
        let mut ctx = Ctx::Pair(a, b);
        self.apply_in(&mut ctx)
    }

    fn apply_in(&self, ctx: &mut Ctx<'_, G>) -> usize
    where
        CrossoverExpr: CrossOp<G>,
    {
        match self {
            Expr::NoOp => 0,

            Expr::Seq(list) => {
                let mut changed = 0;
                for e in list {
                    changed += e.apply_in(ctx);
                }
                changed
            }

            Expr::Prob(p, inner) => {
                if Ctx::<G>::prob(*p) {
                    inner.apply_in(ctx)
                } else {
                    0
                }
            }

            Expr::Select(SelectExpr::All, inner) => match ctx {
                Ctx::One(x) => inner.apply_in(&mut Ctx::One(x)),
                Ctx::Many(xs) => {
                    let mut changed = 0;
                    for x in xs.iter_mut() {
                        changed += inner.apply_in(&mut Ctx::One(x));
                    }
                    changed
                }
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
                Ctx::Many(xs) => {
                    let mut changed = 0;
                    for x in xs.iter_mut() {
                        if pred.test(x) {
                            changed += inner.apply_in(&mut Ctx::One(x));
                        }
                    }
                    changed
                }
                Ctx::Pair(a, b) => {
                    if pred.test(a) {
                        inner.apply_in(&mut Ctx::Pair(*a, *b))
                    } else {
                        0
                    }
                }
                Ctx::ManyPairs(a, b) => {
                    let n = a.len().min(b.len());
                    let mut changed = 0;
                    for i in 0..n {
                        if pred.test(&a[i]) {
                            changed += inner.apply_in(&mut Ctx::Pair(&mut a[i], &mut b[i]));
                        }
                    }
                    changed
                }
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
                Ctx::Many(xs) => {
                    let mut changed = 0;
                    for x in xs.iter_mut() {
                        if pred.test(x) {
                            changed += inner.apply_in(&mut Ctx::One(x));
                        }
                    }
                    changed
                }
                Ctx::Pair(a, b) => {
                    if pred.test(a) {
                        inner.apply_in(&mut Ctx::Pair(*a, *b))
                    } else {
                        0
                    }
                }
                Ctx::ManyPairs(a, b) => {
                    let n = a.len().min(b.len());
                    let mut changed = 0;
                    for i in 0..n {
                        if pred.test(&a[i]) {
                            changed += inner.apply_in(&mut Ctx::Pair(&mut a[i], &mut b[i]));
                        }
                    }
                    changed
                }
            },

            Expr::Mut(f) => match ctx {
                Ctx::One(x) => f.apply(*x),
                Ctx::Many(xs) => xs.iter_mut().map(|x| f.apply(x)).sum(),
                Ctx::Pair(a, b) => f.apply(*a) + f.apply(*b),
                Ctx::ManyPairs(a, b) => {
                    let n = a.len().min(b.len());
                    let mut changed = 0;
                    for i in 0..n {
                        changed += f.apply(&mut a[i]);
                        changed += f.apply(&mut b[i]);
                    }
                    changed
                }
            },

            Expr::Cross(kind, _lhs, _rhs) => match ctx {
                Ctx::Pair(a, b) => kind.op().apply_on_pair(*a, *b),
                Ctx::ManyPairs(a, b) => kind.op().apply_on_slices(*a, *b),
                // Crossing doesn’t make sense on One/Many; treat as no-op.
                _ => 0,
            },
        }
    }
}

#[allow(dead_code)]
pub mod dsl {

    use super::*;
    use crate::expr::{IntoMut, NumericMutateExpr};
    use radiate_core::chromosomes::gene::HasNumericSlot;
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

    pub fn mutate<G, M: IntoMut<G>>(m: M) -> Expr<G> {
        Expr::Mut(m.into_mut())
    }

    pub fn cross_one_point<G>() -> Expr<G> {
        Expr::Cross(
            Xover::new(CrossoverExpr::OnePoint),
            Arc::new(Expr::NoOp),
            Arc::new(Expr::NoOp),
        )
    }

    pub fn cross_two_point<G>() -> Expr<G> {
        Expr::Cross(
            Xover::new(CrossoverExpr::TwoPoint),
            Arc::new(Expr::NoOp),
            Arc::new(Expr::NoOp),
        )
    }

    pub fn mutate_uniform<G: HasNumericSlot + 'static>(range: Range<f32>) -> Expr<G> {
        mutate(NumericMutateExpr::Uniform(range))
    }

    pub fn mutate_gaussian<G: HasNumericSlot + 'static>(mean: f32, std_dev: f32) -> Expr<G> {
        mutate(NumericMutateExpr::Gaussian(mean, std_dev))
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

        // Expr: with p=1.0 select all, apply uniform jitter ±0.25
        let expr: Expr<FloatGene> = prob(1.0, all(mutate_uniform(-1.0..1.0)));

        // run on chromosome
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
            prob(0.5, index(0, mutate(NumericMutateExpr::Uniform(-0.3..0.3)))),
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
        let expr: Expr<FloatGene> = genes().all().cross_two_point().build();

        let changed = expr.apply_pairs(a.genes_mut(), b.genes_mut());
        println!(
            "Changed: {:?}",
            a.iter().map(|g| g.allele()).collect::<Vec<_>>()
        );
        println!(
            "Changed: {:?}",
            b.iter().map(|g| g.allele()).collect::<Vec<_>>()
        );
        assert!(changed > 0);
    }
}
