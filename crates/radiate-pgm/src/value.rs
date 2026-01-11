use crate::{Domain, FactorSpec, Variable};
use radiate_gp::NodeValue;

#[derive(Clone, Debug, PartialEq)]
pub enum PgmValue {
    Variable(Variable),
    Factor(FactorSpec),
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

impl From<Variable> for NodeValue<PgmValue> {
    fn from(value: Variable) -> Self {
        match value.domain {
            Domain::Discrete(k) => NodeValue::Bounded(PgmValue::Variable(value), k.into()),
            Domain::Real => NodeValue::Unbound(PgmValue::Variable(value)),
        }
    }
}

impl From<FactorSpec> for NodeValue<PgmValue> {
    fn from(value: FactorSpec) -> NodeValue<PgmValue> {
        NodeValue::Unbound(PgmValue::Factor(value))
    }
}
