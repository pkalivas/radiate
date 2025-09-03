use crate::expr::{
    CrossoverExpr, Expr, FusedExpr, MutFn, MutateExpr, NumericCrossoverExpr, NumericMutateExpr,
    Pred, SelectExpr, Xover,
};
use radiate_core::{
    ArithmeticGene, Gene,
    chromosomes::gene::{HasNumericSlot, NumericAllele, NumericGene},
};
use std::{ops::Range, sync::Arc};

type Wrap<G> = Box<dyn Fn(Expr<G>) -> Expr<G> + Send + Sync>;

pub struct ExprBuilder<G: Gene> {
    stages: Vec<Expr<G>>,
    pending: Vec<Wrap<G>>,
}

#[allow(dead_code)]
pub fn genes<G: Gene>() -> ExprBuilder<G> {
    ExprBuilder::new()
}

impl<G: Gene> Default for ExprBuilder<G> {
    fn default() -> Self {
        Self {
            stages: Vec::new(),
            pending: Vec::new(),
        }
    }
}

#[allow(dead_code)]
impl<G: Gene> ExprBuilder<G> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(mut self) -> Self {
        self.pending.push(Box::new(|inner| {
            Expr::Select(SelectExpr::All, Arc::new(inner))
        }));
        self
    }

    pub fn some<F>(mut self, pred: F) -> Self
    where
        G: Clone + 'static,
        F: Fn(&G) -> bool + Send + Sync + 'static,
    {
        let p = Pred::new(move |g| pred(g));
        self.pending.push(Box::new(move |inner| {
            Expr::Select(SelectExpr::Some(p.clone()), Arc::new(inner))
        }));
        self
    }

    pub fn index(mut self, i: usize) -> Self {
        self.pending
            .push(Box::new(move |inner| Expr::Index(i, Arc::new(inner))));
        self
    }

    pub fn filter<F>(mut self, pred: F) -> Self
    where
        G: Clone + 'static,
        F: Fn(&G) -> bool + Send + Sync + 'static,
    {
        let p = Pred::new(move |g| pred(g));
        self.pending.push(Box::new(move |inner| {
            Expr::Filter(p.clone(), Arc::new(inner))
        }));
        self
    }

    pub fn prob(mut self, p: f32) -> Self {
        self.pending
            .push(Box::new(move |inner| Expr::Prob(p, Arc::new(inner))));
        self
    }

    pub fn map<F>(mut self, f: fn(&mut G) -> usize) -> Self
    where
        G: Clone + 'static,
    {
        let mapper = MutFn::from_fn_ptr(f);
        let expr = Expr::Mut(mapper.clone());
        self.pending.push(Box::new(move |_| expr.clone()));
        self
    }

    pub fn mutate<M: Into<MutFn<G>>>(mut self, m: M) -> Self {
        self.push_terminal(Expr::Mut(m.into()));
        self
    }

    pub fn cross<C: Into<Xover<G>>>(mut self, kind: C) -> Self
    where
        C: Send + Sync + 'static,
    {
        self.push_terminal(Expr::Cross(
            kind.into(),
            Arc::new(Expr::NoOp),
            Arc::new(Expr::NoOp),
        ));
        self
    }

    pub fn then(mut self) -> Self {
        if !self.pending.is_empty() {
            self.push_terminal(Expr::NoOp);
        }
        self
    }

    pub fn one_point_cross(self) -> Self {
        self.cross(CrossoverExpr::OnePoint)
    }

    pub fn two_point_cross(self) -> Self {
        self.cross(CrossoverExpr::TwoPoint)
    }

    pub fn swap_cross(self) -> Self {
        self.cross(CrossoverExpr::Swap)
    }

    pub fn blend_cross(self, factor: f32) -> Self
    where
        G: HasNumericSlot,
    {
        self.cross(NumericCrossoverExpr::Blend(factor))
    }

    pub fn intermediate_cross(self, alpha: f32) -> Self
    where
        G: HasNumericSlot,
    {
        self.cross(NumericCrossoverExpr::Intermediate(alpha))
    }

    pub fn mean_cross(self) -> Self
    where
        G: HasNumericSlot,
    {
        self.cross(NumericCrossoverExpr::Mean)
    }

    pub fn uniform_mutate(self, range: Range<f32>) -> Self
    where
        G: NumericGene,
        G::Allele: NumericAllele,
    {
        self.mutate(NumericMutateExpr::Uniform(range))
    }

    pub fn gaussian_mutate(self, mean: f32, std_dev: f32) -> Self
    where
        G: NumericGene,
        G::Allele: NumericAllele,
    {
        self.mutate(NumericMutateExpr::Gaussian(mean, std_dev))
    }

    pub fn jitter_mutate(self, amount: f32) -> Self
    where
        G: NumericGene,
        G::Allele: NumericAllele,
    {
        self.mutate(NumericMutateExpr::Jitter(amount))
    }

    pub fn inversion_mutate(self) -> Self
    where
        G: 'static,
    {
        self.mutate(MutateExpr::Invert)
    }

    pub fn build(mut self) -> Expr<G> {
        if !self.pending.is_empty() {
            self.push_terminal(Expr::NoOp);
        }

        match self.stages.len() {
            0 => Expr::Seq(vec![]),
            1 => self.stages.pop().unwrap(),
            _ => Expr::Seq(self.stages),
        }
    }

    fn push_terminal(&mut self, terminal: Expr<G>) {
        let expr = fuse_expr(self.pending.drain(..).rfold(terminal, |acc, w| (w)(acc)));
        self.stages.push(expr);
    }
}

pub fn fuse_expr<G: Gene>(expr: Expr<G>) -> Expr<G> {
    match expr {
        Expr::Select(SelectExpr::All, inner) => match inner.as_ref() {
            Expr::Prob(p, inner) => match inner.as_ref() {
                Expr::Mut(m) => Expr::Fused(FusedExpr::Mutate(Some(*p), (*m).clone())),
                _ => Expr::Fused(FusedExpr::None),
            },

            _ => Expr::Fused(FusedExpr::None),
        },
        _ => Expr::Fused(FusedExpr::None),
    }
}
