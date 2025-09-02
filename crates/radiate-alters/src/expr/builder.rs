use crate::expr::{CrossOp, CrossoverExpr, Expr, IntoMut, MutFn, Pred, SelectExpr, Xover};
use std::sync::Arc;

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

    pub fn mutate<M: IntoMut<G>>(mut self, m: M) -> Self {
        self.push_terminal(Expr::Mut(m.into_mut()));
        self
    }

    pub fn cross<C: CrossOp<G>>(mut self, kind: C) -> Self
    where
        C: Send + Sync + 'static,
    {
        self.push_terminal(Expr::Cross(
            Xover::new(kind),
            Arc::new(Expr::Mut(MutFn::new(|_: &mut G| 0))),
            Arc::new(Expr::Mut(MutFn::new(|_: &mut G| 0))),
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

    pub fn cross_one_point(self) -> Self {
        self.cross(CrossoverExpr::OnePoint)
    }

    pub fn cross_two_point(self) -> Self {
        self.cross(CrossoverExpr::TwoPoint)
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
