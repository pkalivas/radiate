use crate::{Domain, FactorSpec, Variable};
use radiate_gp::{NodeValue, Op};

#[derive(Clone, Debug, PartialEq)]
pub enum PgmValue {
    Variable(Variable),
    Factor(FactorSpec),
}

impl Default for PgmValue {
    fn default() -> Self {
        PgmValue::Variable(Variable::default())
    }
}

impl From<PgmValue> for NodeValue<PgmValue> {
    fn from(value: PgmValue) -> Self {
        match &value {
            PgmValue::Variable(var) => match var.domain {
                Domain::Discrete(k) => NodeValue::Bounded(value, k.into()),
                Domain::Real => NodeValue::Unbound(value),
            },
            PgmValue::Factor(_) => NodeValue::Unbound(value),
        }
    }
}
