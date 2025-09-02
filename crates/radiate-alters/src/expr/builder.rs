use radiate_core::chromosomes::gene::HasNumericSlot;

use crate::expr::{
    CrossoverExpr, Expr, MutFn, MutateExpr, NumericCrossoverExpr, NumericMutateExpr, Pred,
    SelectExpr, Xover,
};
use std::{ops::Range, sync::Arc};

type Wrap<G> = Box<dyn Fn(Expr<G>) -> Expr<G> + Send + Sync>;

pub struct ExprBuilder<G> {
    stages: Vec<Expr<G>>,
    pending: Vec<Wrap<G>>,
}

#[allow(dead_code)]
pub fn genes<G>() -> ExprBuilder<G> {
    ExprBuilder::new()
}

impl<G> Default for ExprBuilder<G> {
    fn default() -> Self {
        Self {
            stages: Vec::new(),
            pending: Vec::new(),
        }
    }
}

#[allow(dead_code)]
impl<G> ExprBuilder<G> {
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

    pub fn map<F>(mut self, f: F) -> Self
    where
        G: Clone + 'static,
        F: Fn(&mut G) -> usize + Send + Sync + 'static,
    {
        let mapper = MutFn::new(f);
        let expr = Expr::Mut(mapper.clone());
        self.pending.push(Box::new(move |_| expr.clone()));
        self
    }

    pub fn map_each(mut self, inner: impl Into<Expr<G>>) -> Self
    where
        G: 'static,
    {
        let inner = Arc::new(inner.into());
        self.pending
            .push(Box::new(move |_| Expr::MapEach(inner.clone())));
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

    /// Start a new stage in the top-level sequence.
    /// (No-op if a stage hasn't been closed by a terminal; you typically call
    /// `then()` after a terminal like `.mutate(...)`.)
    pub fn then(mut self) -> Self {
        // If a user calls then() with pending wrappers and no terminal,
        // we close the stage with a no-op Map so the structure is preserved.
        if !self.pending.is_empty() {
            self.push_terminal(Expr::Mut(MutFn::new(|_: &mut G| 0)));
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
        G: HasNumericSlot + 'static,
    {
        self.mutate(NumericMutateExpr::Uniform(range))
    }

    pub fn gausian_mutate(self, mean: f32, std_dev: f32) -> Self
    where
        G: HasNumericSlot + 'static,
    {
        self.mutate(NumericMutateExpr::Gaussian(mean, std_dev))
    }

    pub fn jitter_mutate(self, amount: f32) -> Self
    where
        G: HasNumericSlot + 'static,
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
            self.push_terminal(Expr::Mut(MutFn::new(|_: &mut G| 0)));
        }

        match self.stages.len() {
            0 => Expr::Seq(vec![]),
            1 => self.stages.pop().unwrap(),
            _ => Expr::Seq(self.stages),
        }
    }

    fn push_terminal(&mut self, terminal: Expr<G>) {
        let expr = self.pending.drain(..).rfold(terminal, |acc, w| (w)(acc));
        self.stages.push(expr);
    }
}
