use crate::Arity;
use crate::ops::op_names;
use crate::ops::operation::Op;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, de::Deserializer, ser::Serializer};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[serde(tag = "type", content = "data")]
enum OpVariant<T> {
    Fn {
        name: String,
        arity: Arity,
    },
    Var {
        name: String,
        index: usize,
        card: Option<usize>,
    },
    Const {
        name: String,
        value: T,
    },
    Value {
        name: String,
        arity: Arity,
        value: radiate_utils::Value<T>,
    },
}

#[cfg(feature = "serde")]
impl<T> Serialize for Op<T>
where
    T: Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let variant = OpVariant::from(self.clone());
        variant.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Op<f32> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let variant = OpVariant::<f32>::deserialize(deserializer)?;
        Result::from(variant).map_err(|e| {
            serde::de::Error::custom(format!("Failed to convert OpVariant to Op: {}", e))
        })
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Op<bool> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let variant = OpVariant::<bool>::deserialize(deserializer)?;
        Result::from(variant).map_err(|e| {
            serde::de::Error::custom(format!("Failed to convert OpVariant to Op: {}", e))
        })
    }
}

impl<T: Clone> From<Op<T>> for OpVariant<T> {
    fn from(op: Op<T>) -> Self {
        match op {
            Op::Fn(name, arity, _) => OpVariant::Fn {
                name: name.to_string(),
                arity,
            },
            Op::Var(name, index, card) => OpVariant::Var {
                name: name.to_string(),
                index,
                card,
            },
            Op::Const(name, value) => OpVariant::Const {
                name: name.to_string(),
                value,
            },
            Op::Value(name, arity, value, ..) => OpVariant::Value {
                name: name.to_string(),
                arity,
                value: value.data().clone(),
            },
        }
    }
}

impl From<OpVariant<f32>> for Result<Op<f32>, serde::de::value::Error> {
    fn from(variant: OpVariant<f32>) -> Self {
        match variant {
            OpVariant::Fn { name, .. } => {
                let name: &'static str = Box::leak(name.into_boxed_str());
                match name {
                    op_names::ADD => Ok(Op::add()),
                    op_names::SUB => Ok(Op::sub()),
                    op_names::MUL => Ok(Op::mul()),
                    op_names::DIV => Ok(Op::div()),
                    op_names::SUM => Ok(Op::sum()),
                    op_names::PROD => Ok(Op::prod()),
                    op_names::DIFF => Ok(Op::diff()),
                    op_names::NEG => Ok(Op::neg()),
                    op_names::POW => Ok(Op::pow()),
                    op_names::SQRT => Ok(Op::sqrt()),
                    op_names::ABS => Ok(Op::abs()),
                    op_names::EXP => Ok(Op::exp()),
                    op_names::LOG => Ok(Op::log()),
                    op_names::SIN => Ok(Op::sin()),
                    op_names::COS => Ok(Op::cos()),
                    op_names::TAN => Ok(Op::tan()),
                    op_names::CEIL => Ok(Op::ceil()),
                    op_names::FLOOR => Ok(Op::floor()),
                    op_names::MAX => Ok(Op::max()),
                    op_names::MIN => Ok(Op::min()),
                    op_names::SIGMOID => Ok(Op::sigmoid()),
                    op_names::TANH => Ok(Op::tanh()),
                    op_names::RELU => Ok(Op::relu()),
                    op_names::LEAKY_RELU => Ok(Op::leaky_relu()),
                    op_names::ELU => Ok(Op::elu()),
                    op_names::LINEAR => Ok(Op::linear()),
                    op_names::MISH => Ok(Op::mish()),
                    op_names::SWISH => Ok(Op::swish()),
                    op_names::SOFTPLUS => Ok(Op::softplus()),
                    op_names::IDENTITY => Ok(Op::identity()),
                    op_names::LOGSUMEXP => Ok(Op::logsumexp()),
                    _ => Err(serde::de::Error::custom(format!(
                        "Unknown function name: {}",
                        name
                    ))),
                }
            }
            OpVariant::Var { name, index, card } => {
                let name = radiate_utils::intern!(name);
                Ok(Op::Var(name, index, card))
            }
            OpVariant::Const { name, value } => {
                let name = radiate_utils::intern!(name);
                Ok(Op::Const(name, value))
            }
            OpVariant::Value {
                name,
                arity: _,
                value,
            } => match name.as_str() {
                "w" => {
                    let weight = Op::weight_with(value.as_scalar().cloned().unwrap_or(0.0));
                    Ok(weight)
                }
                _ => {
                    return Err(serde::de::Error::custom(format!(
                        "Unknown mutable constant name: {}",
                        name
                    )));
                }
            },
        }
    }
}

impl From<OpVariant<bool>> for Result<Op<bool>, serde::de::value::Error> {
    fn from(variant: OpVariant<bool>) -> Self {
        match variant {
            OpVariant::Fn { name, .. } => {
                let name: &'static str = Box::leak(name.into_boxed_str());
                match name {
                    op_names::AND => Ok(Op::and()),
                    op_names::OR => Ok(Op::or()),
                    op_names::NOT => Ok(Op::not()),
                    op_names::XOR => Ok(Op::xor()),
                    op_names::EQ => Ok(Op::eq()),
                    op_names::NE => Ok(Op::ne()),
                    op_names::GT => Ok(Op::gt()),
                    op_names::GE => Ok(Op::ge()),
                    op_names::LT => Ok(Op::lt()),
                    op_names::LE => Ok(Op::le()),
                    op_names::IF_ELSE => Ok(Op::if_else()),
                    op_names::AND_THEN => Ok(Op::and_then()),
                    op_names::OR_ELSE => Ok(Op::or_else()),
                    op_names::NAND => Ok(Op::nand()),
                    op_names::NOR => Ok(Op::nor()),
                    op_names::XNOR => Ok(Op::xnor()),
                    op_names::IMPLIES => Ok(Op::implies()),
                    op_names::IFF => Ok(Op::iff()),
                    _ => Err(serde::de::Error::custom(format!(
                        "Unknown boolean function name: {}",
                        name
                    ))),
                }
            }
            OpVariant::Value { name, .. } => {
                return Err(serde::de::Error::custom(format!(
                    "Mutable constants are not supported for boolean ops: {}",
                    name
                )));
            }
            OpVariant::Var { name, index, card } => {
                let name = radiate_utils::intern!(name);
                Ok(Op::Var(name, index, card))
            }
            OpVariant::Const { name, value } => {
                let name = radiate_utils::intern!(name);
                Ok(Op::Const(name, value))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Eval;

    use super::*;
    use serde_json;

    #[test]
    fn test_serialize_deserialize_const() {
        let op = Op::constant(42.0);
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: Op<f32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op.name(), deserialized.name());
        assert_eq!(op.arity(), deserialized.arity());
    }

    #[test]
    fn test_serialize_deserialize_fn() {
        let op = Op::add();
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: Op<f32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op.name(), deserialized.name());
        assert_eq!(op.arity(), deserialized.arity());

        // Test that the deserialized function works
        let inputs = [1.0, 2.0];
        assert_eq!(op.eval(&inputs), deserialized.eval(&inputs));
    }

    #[test]
    fn test_serialize_deserialize_var() {
        let op = Op::<f32>::var(42);
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: Op<f32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op.name(), deserialized.name());
        assert_eq!(op.arity(), deserialized.arity());
    }

    #[test]
    fn test_serialize_deserialize_mutable_const() {
        let op = Op::weight();
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: Op<f32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op.name(), deserialized.name());
        assert_eq!(op.arity(), deserialized.arity());
    }
}
