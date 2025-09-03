use radiate_core::{FloatGene, Gene, random_provider};

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
pub enum Expr<G: Gene> {
    #[default]
    NoOp,
    Index(usize, E<G>),
    Mut(MutFn<G>),
    Cross(Xover<G>, E<G>, E<G>),
    Filter(Pred<G>, E<G>),
    Prob(f32, E<G>),
    Seq(Vec<Expr<G>>),
    Select(SelectExpr<G>, E<G>),
    Fused(FusedExpr<G>),
}

enum Ctx<'a, G> {
    One(&'a mut G),
    Many(&'a mut [G]),
    Pair(&'a mut G, &'a mut G),
    ManyPairs(&'a mut [G], &'a mut [G]),
}

impl<'a, G> Ctx<'a, G> {
    #[inline(always)]
    fn prob(p: f32) -> bool {
        p > 0.0 && p <= 1.0 && random_provider::random::<f32>() < p
    }
}

impl<G: Gene> Expr<G> {
    #[inline(always)]
    pub fn apply(&self, x: &mut G) -> usize {
        match self {
            Expr::NoOp => 0,
            Expr::Prob(p, inner) => {
                if Ctx::<G>::prob(*p) {
                    inner.apply(x)
                } else {
                    0
                }
            }
            Expr::Select(SelectExpr::All, inner) => inner.apply(x),
            Expr::Select(SelectExpr::Some(pred), inner) => {
                if pred.test(x) {
                    inner.apply(x)
                } else {
                    0
                }
            }
            Expr::Index(_, inner) => inner.apply(x),
            Expr::Filter(pred, inner) => {
                if pred.test(x) {
                    inner.apply(x)
                } else {
                    0
                }
            }
            Expr::Mut(f) => f.apply(x),
            Expr::Seq(list) => list.iter().map(|e| e.apply(x)).sum(),
            Expr::Cross(_, _, _) => 0,
            Expr::Fused(fused) => match fused {
                FusedExpr::Mutate(prob, f) => prob
                    .map(|rate| if Ctx::<G>::prob(rate) { f.apply(x) } else { 0 })
                    .unwrap_or_else(|| f.apply(x)),
                FusedExpr::None => 0,
            },
        }
    }

    #[inline(always)]
    pub fn apply_slice(&self, xs: &mut [G]) -> usize {
        match self {
            Expr::NoOp => 0,
            Expr::Prob(p, inner) => {
                if Ctx::<G>::prob(*p) {
                    inner.apply_slice(xs)
                } else {
                    0
                }
            }
            Expr::Select(SelectExpr::All, inner) => {
                let mut acc = 0;
                for x in xs {
                    acc += inner.apply(x);
                }
                acc
            }
            Expr::Select(SelectExpr::Some(pred), inner) => xs
                .iter_mut()
                .filter(|x| pred.test(x))
                .map(|x| inner.apply(x))
                .sum(),
            Expr::Index(i, inner) => {
                if *i < xs.len() {
                    inner.apply(&mut xs[*i])
                } else {
                    0
                }
            }
            Expr::Filter(pred, inner) => xs
                .iter_mut()
                .filter(|x| pred.test(x))
                .map(|x| inner.apply(x))
                .sum(),
            Expr::Mut(f) => f.apply_slice(xs),
            Expr::Seq(list) => list.iter().map(|e| e.apply_slice(xs)).sum(),
            Expr::Cross(_, _, _) => 0,
            Expr::Fused(fused) => match fused {
                FusedExpr::Mutate(prob, f) => prob
                    .map(|rate| {
                        xs.iter_mut()
                            .filter(|_| Ctx::<G>::prob(rate))
                            .map(|gene| f.apply(gene))
                            .sum()
                    })
                    .unwrap_or(0),
                FusedExpr::None => 0,
            },
        }
    }

    #[inline(always)]
    pub fn apply_pair(&self, a: &mut G, b: &mut G) -> usize {
        match self {
            Expr::NoOp => 0,
            Expr::Prob(p, inner) => {
                if Ctx::<G>::prob(*p) {
                    inner.apply_pair(a, b)
                } else {
                    0
                }
            }
            Expr::Select(SelectExpr::All, inner) => inner.apply_pair(a, b),
            Expr::Select(SelectExpr::Some(pred), inner) | Expr::Filter(pred, inner) => {
                if pred.test(a) {
                    inner.apply_pair(a, b)
                } else {
                    0
                }
            }
            Expr::Index(_, inner) => inner.apply_pair(a, b),
            Expr::Mut(f) => f.apply(a) + f.apply(b),
            Expr::Seq(list) => list.iter().map(|e| e.apply_pair(a, b)).sum(),
            Expr::Cross(kind, lhs, rhs) => {
                let mut acc = 0;
                acc += lhs.apply(a);
                acc += rhs.apply(b);
                acc + kind.op().apply_on_pair(a, b)
            }
            Expr::Fused(fused) => match fused {
                FusedExpr::Mutate(prob, f) => {
                    let mut acc = 0;
                    if let Some(p) = prob {
                        if Ctx::<G>::prob(*p) {
                            acc += f.apply(a);
                            acc += f.apply(b);
                        }
                    }
                    acc
                }
                FusedExpr::None => 0,
            },
        }
    }

    #[inline(always)]
    fn apply_slice_pairs(&self, a: &mut [G], b: &mut [G]) -> usize {
        match self {
            Expr::NoOp => 0,
            Expr::Prob(p, inner) => {
                if Ctx::<G>::prob(*p) {
                    inner.apply_slice_pairs(a, b)
                } else {
                    0
                }
            }
            Expr::Select(SelectExpr::All, inner) => a
                .iter_mut()
                .zip(b.iter_mut())
                .map(|(x, y)| inner.apply_pair(x, y))
                .sum(),
            Expr::Select(SelectExpr::Some(pred), inner) | Expr::Filter(pred, inner) => a
                .iter_mut()
                .zip(b.iter_mut())
                .filter(|(x, _)| pred.test(x))
                .map(|(x, y)| inner.apply_pair(x, y))
                .sum(),
            Expr::Index(i, inner) => {
                let n = a.len().min(b.len());
                if *i < n {
                    inner.apply_pair(&mut a[*i], &mut b[*i])
                } else {
                    0
                }
            }
            Expr::Mut(f) => a
                .iter_mut()
                .zip(b.iter_mut())
                .map(|(x, y)| f.apply(x) + f.apply(y))
                .sum(),
            Expr::Seq(list) => list.iter().map(|e| e.apply_slice_pairs(a, b)).sum(),
            Expr::Cross(kind, lhs, rhs) => {
                let mut acc = 0;
                acc += lhs.apply_slice(a);
                acc += rhs.apply_slice(b);
                acc + kind.op().apply_on_slices(a, b)
            }
            Expr::Fused(fused) => match fused {
                FusedExpr::Mutate(prob, f) => {
                    let mut acc = 0;
                    for x in a {
                        if let Some(p) = prob {
                            if Ctx::<G>::prob(*p) {
                                acc += f.apply(x);
                            }
                        } else if f.apply(x) > 0 {
                            acc += 1;
                        }
                    }
                    acc
                }
                FusedExpr::None => 0,
            },
        }
    }
}

#[allow(dead_code)]
pub mod dsl {

    use super::*;
    use crate::expr::{MutateExpr, NumericCrossoverExpr, NumericMutateExpr, fuse_expr};
    use radiate_core::{
        ArithmeticGene, Gene,
        chromosomes::gene::{HasNumericSlot, NumericAllele, NumericGene},
    };
    use std::ops::{Mul, Range};

    pub fn all<G: Gene>(inner: Expr<G>) -> Expr<G> {
        Expr::Select(SelectExpr::All, Arc::new(inner))
    }

    pub fn index<G: Gene>(i: usize, inner: Expr<G>) -> Expr<G> {
        Expr::Index(i, Arc::new(inner))
    }

    pub fn filter<G: Gene>(
        pred: impl Fn(&G) -> bool + Send + Sync + 'static,
        inner: Expr<G>,
    ) -> Expr<G> {
        Expr::Filter(Pred::new(pred), Arc::new(inner))
    }

    pub fn prob<G: Gene>(p: f32, inner: Expr<G>) -> Expr<G> {
        Expr::Prob(p, Arc::new(inner))
    }

    pub fn seq<G: Gene>(xs: impl Into<Vec<Expr<G>>>) -> Expr<G> {
        Expr::Seq(xs.into())
    }

    pub fn prob_each<G: Gene>(p: f32, inner: Expr<G>) -> Expr<G> {
        all(Expr::Prob(p, Arc::new(inner)))
    }

    pub fn mutate<G: Gene, M: Into<MutFn<G>>>(m: M) -> Expr<G> {
        Expr::Mut(m.into())
    }

    pub fn cross<G: Gene>(
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
    pub fn one_point_cross<G: Gene>() -> Expr<G> {
        cross(CrossoverExpr::OnePoint, None, None)
    }

    pub fn two_point_cross<G: Gene>() -> Expr<G> {
        cross(CrossoverExpr::TwoPoint, None, None)
    }

    pub fn blend_cross<G: ArithmeticGene + HasNumericSlot + 'static>(alpha: f32) -> Expr<G> {
        cross(NumericCrossoverExpr::Blend(alpha), None, None)
    }

    pub fn intermediate_cross<G: ArithmeticGene + HasNumericSlot + 'static>(alpha: f32) -> Expr<G> {
        cross(NumericCrossoverExpr::Intermediate(alpha), None, None)
    }

    pub fn mean_cross<G: ArithmeticGene + HasNumericSlot + 'static>() -> Expr<G> {
        cross(NumericCrossoverExpr::Mean, None, None)
    }

    pub fn swap_cross<G: Gene>() -> Expr<G> {
        cross(CrossoverExpr::Swap, None, None)
    }

    /// Mutation operations
    pub fn uniform_mutate<G>(range: Range<f32>) -> Expr<G>
    where
        G: NumericGene,
        G::Allele: NumericAllele,
    {
        mutate(NumericMutateExpr::Uniform(range))
    }

    pub fn gaussian_mutate<G>(mean: f32, std_dev: f32) -> Expr<G>
    where
        G: NumericGene,
        G::Allele: NumericAllele,
    {
        mutate(NumericMutateExpr::Gaussian(mean, std_dev))
    }

    pub fn jitter_mutate<G>(amount: f32) -> Expr<G>
    where
        G: NumericGene,
        G::Allele: NumericAllele,
    {
        mutate(NumericMutateExpr::Jitter(amount))
    }

    pub fn inversion_mutate<G: Gene + 'static>() -> Expr<G> {
        mutate(MutateExpr::Invert)
    }

    pub fn gaussian_by_bounds_generic<G>() -> Expr<G>
    where
        G: NumericGene + HasNumericSlot + 'static,
        G::Allele: NumericAllele + Mul<Output = f64> + Into<f64> + From<f64>,
    {
        mutate(MutFn::named_fn("gauss-by-bounds", |g: &mut G| {
            // let min = (*g.min()).into() as f64;
            // let max = (*g.max()).into() as f64;
            // let mean = (*g.allele()).into() as f64;
            // let std = (max - min) * 0.25_f64;
            super::super::mutate::apply_gaussian(g)
        }))
    }

    pub fn build<G: Gene>(expr: Expr<G>) -> Expr<G> {
        fuse_expr(expr)
    }
}

impl<G: Gene> Debug for Expr<G> {
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
            all(mutate(MutFn::named_fn(
                "jitter±0.1",
                |g: &mut FloatGene| {
                    *g.allele_mut() += radiate_core::random_provider::range(-0.1..0.1);
                    1
                },
            ))),
            prob(
                0.5,
                index(
                    0,
                    mutate(MutFn::named_fn("kick0", |g: &mut FloatGene| {
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

        // // Cross whole chromosome with a one-point crossover
        // let _: Expr<FloatGene> = genes()
        //     .two_point_cross()
        //     .map_each(prob(0.5, mean_cross()))
        //     .build();

        // let expr = genes()
        //     .all()
        //     .prob(
        //         0.3, // 30% of loci
        //         cross(
        //             CrossoverExpr::TwoPoint,
        //             Some(jitter_mutate(0.05)), // only parent A
        //             Some(jitter_mutate(0.05)), // only parent B
        //         ),
        //     )
        //     .build();

        // let changed = expr.apply_slice_pairs(a.genes_mut(), b.genes_mut());
        // println!(
        //     "Changed: {:?}",
        //     a.iter().map(|g| g.allele()).collect::<Vec<_>>()
        // );
        // println!(
        //     "Changed: {:?}",
        //     b.iter().map(|g| g.allele()).collect::<Vec<_>>()
        // );

        // println!("Tree: {:?}", expr.dump_tree());
        // assert!(changed > 0);
    }
}

#[derive(Debug, Clone)]
pub enum FusedExpr<G> {
    Mutate(Option<f32>, MutFn<G>),
    None,
}
