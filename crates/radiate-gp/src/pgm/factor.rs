use crate::{Arity, NodeValue, Op};

#[derive(Clone, Debug, PartialEq)]
pub enum FactorType {
    GaussLin2,
    Gauss1,
    Logp,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PgmOp<T> {
    Variable(&'static str, usize, Option<usize>),
    Factor(FactorType),
    Op(Op<T>),
}

impl PgmOp<f32> {
    pub fn logprob() -> Self {
        PgmOp::Factor(FactorType::Logp)
    }

    pub fn gauss_lin2() -> Self {
        PgmOp::Factor(FactorType::GaussLin2)
    }

    pub fn gauss1() -> Self {
        PgmOp::Factor(FactorType::Gauss1)
    }

    pub fn variable(name: &'static str, index: usize, domain: Option<usize>) -> Self {
        PgmOp::Variable(name, index, domain)
    }
}

impl<T: Clone> From<PgmOp<T>> for NodeValue<PgmOp<T>> {
    fn from(value: PgmOp<T>) -> Self {
        let other = value.clone();
        match value {
            PgmOp::Variable(_, _, card) => match card {
                Some(_) => NodeValue::Bounded(other, Arity::Zero),
                None => NodeValue::Bounded(other, Arity::Zero),
            },
            PgmOp::Factor(_) => NodeValue::Unbound(other),
            PgmOp::Op(_) => NodeValue::Unbound(other),
        }
    }
}

impl<T: Default + Clone> From<&PgmOp<T>> for Op<T> {
    fn from(value: &PgmOp<T>) -> Op<T> {
        match value {
            PgmOp::Variable(name, idx, domain) => Op::Var(name, *idx, domain.clone()),
            PgmOp::Op(op) => op.clone(),
            _ => Op::default(),
        }
    }
}
