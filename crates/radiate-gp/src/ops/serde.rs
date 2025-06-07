use crate::Arity;
use crate::ops::operation::Op;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum OpVariant<T> {
    Fn {
        name: String,
        arity: Arity,
    },
    Var {
        name: String,
        index: usize,
    },
    Const {
        name: String,
        value: T,
    },
    MutableConst {
        name: String,
        arity: Arity,
        value: T,
    },
    Value {
        name: String,
        arity: Arity,
        value: T,
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
        let variant = match self {
            Op::Fn(name, arity, _) => OpVariant::Fn {
                name: name.to_string(),
                arity: *arity,
            },
            Op::Var(name, index) => OpVariant::Var {
                name: name.to_string(),
                index: *index,
            },
            Op::Const(name, value) => OpVariant::Const {
                name: name.to_string(),
                value: value.clone(),
            },
            Op::MutableConst {
                name, arity, value, ..
            } => OpVariant::MutableConst {
                name: name.to_string(),
                arity: *arity,
                value: value.clone(),
            },
            Op::Value(name, arity, value, _) => OpVariant::Value {
                name: name.to_string(),
                arity: *arity,
                value: value.clone(),
            },
        };
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
        match variant {
            OpVariant::Fn { name, .. } => {
                let name: &'static str = Box::leak(name.into_boxed_str());
                match name {
                    "add" => Ok(Op::add()),
                    "sub" => Ok(Op::sub()),
                    "mul" => Ok(Op::mul()),
                    "div" => Ok(Op::div()),
                    "sum" => Ok(Op::sum()),
                    "prod" => Ok(Op::prod()),
                    "diff" => Ok(Op::diff()),
                    "neg" => Ok(Op::neg()),
                    "pow" => Ok(Op::pow()),
                    "sqrt" => Ok(Op::sqrt()),
                    "abs" => Ok(Op::abs()),
                    "exp" => Ok(Op::exp()),
                    "log" => Ok(Op::log()),
                    "sin" => Ok(Op::sin()),
                    "cos" => Ok(Op::cos()),
                    "tan" => Ok(Op::tan()),
                    "ceil" => Ok(Op::ceil()),
                    "floor" => Ok(Op::floor()),
                    "max" => Ok(Op::max()),
                    "min" => Ok(Op::min()),
                    "sigmoid" => Ok(Op::sigmoid()),
                    "tanh" => Ok(Op::tanh()),
                    "relu" => Ok(Op::relu()),
                    "l_relu" => Ok(Op::leaky_relu()),
                    "elu" => Ok(Op::elu()),
                    "linear" => Ok(Op::linear()),
                    "mish" => Ok(Op::mish()),
                    "swish" => Ok(Op::swish()),
                    "softplus" => Ok(Op::softplus()),
                    "softmax" => Ok(Op::softmax()),
                    "identity" => Ok(Op::identity()),
                    _ => Err(serde::de::Error::custom(format!(
                        "Unknown function name: {}",
                        name
                    ))),
                }
            }
            OpVariant::Var { name, index } => {
                let name = Box::leak(name.into_boxed_str());
                Ok(Op::Var(name, index))
            }
            OpVariant::Const { name, value } => {
                let name = Box::leak(name.into_boxed_str());
                Ok(Op::Const(name, value))
            }
            OpVariant::MutableConst { name, arity, value } => match name.as_str() {
                "w" => {
                    let weight = Op::weight();
                    match weight {
                        Op::MutableConst {
                            name,
                            arity: w_arity,
                            value: _,
                            supplier: w_supplier,
                            modifier: w_modifier,
                            operation: w_operation,
                        } => {
                            return Ok(Op::MutableConst {
                                name,
                                arity: w_arity.clone(),
                                value: value.clone(),
                                supplier: Arc::clone(&w_supplier),
                                modifier: Arc::clone(&w_modifier),
                                operation: Arc::clone(&w_operation),
                            });
                        }
                        _ => {
                            let name = Box::leak(name.into_boxed_str());
                            let supplier = Arc::new(move || value.clone());
                            let modifier = Arc::new(|v: &f32| v.clone());
                            let operation = Arc::new(|_: &[f32], v: &f32| v.clone());
                            Ok(Op::MutableConst {
                                name,
                                arity,
                                value,
                                supplier,
                                modifier,
                                operation,
                            })
                        }
                    }
                }
                _ => {
                    return Err(serde::de::Error::custom(format!(
                        "Unknown mutable constant name: {}",
                        name
                    )));
                }
            },
            OpVariant::Value { name, arity, value } => {
                let name = Box::leak(name.into_boxed_str());
                Ok(Op::Value(
                    name,
                    arity,
                    value,
                    Arc::new(|_: &[f32], v: &f32| v.clone()),
                ))
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Op<bool> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let variant = OpVariant::<bool>::deserialize(deserializer)?;
        match variant {
            OpVariant::Fn { name, .. } => {
                let name: &'static str = Box::leak(name.into_boxed_str());
                match name {
                    "and" => Ok(Op::and()),
                    "or" => Ok(Op::or()),
                    "not" => Ok(Op::not()),
                    "xor" => Ok(Op::xor()),
                    "eq" => Ok(Op::eq()),
                    "ne" => Ok(Op::ne()),
                    "gt" => Ok(Op::gt()),
                    "ge" => Ok(Op::ge()),
                    "lt" => Ok(Op::lt()),
                    "le" => Ok(Op::le()),
                    "if_else" => Ok(Op::if_else()),
                    "and_then" => Ok(Op::and_then()),
                    "or_else" => Ok(Op::or_else()),
                    "nand" => Ok(Op::nand()),
                    "nor" => Ok(Op::nor()),
                    "xnor" => Ok(Op::xnor()),
                    "implies" => Ok(Op::implies()),
                    "iff" => Ok(Op::iff()),
                    _ => Err(serde::de::Error::custom(format!(
                        "Unknown boolean function name: {}",
                        name
                    ))),
                }
            }
            OpVariant::MutableConst { name, .. } => {
                return Err(serde::de::Error::custom(format!(
                    "Mutable constants are not supported for boolean ops: {}",
                    name
                )));
            }
            OpVariant::Value { name, arity, value } => {
                let name = Box::leak(name.into_boxed_str());
                Ok(Op::Value(
                    name,
                    arity,
                    value,
                    Arc::new(|_: &[bool], v: &bool| v.clone()),
                ))
            }
            OpVariant::Var { name, index } => {
                let name = Box::leak(name.into_boxed_str());
                Ok(Op::Var(name, index))
            }
            OpVariant::Const { name, value } => {
                let name = Box::leak(name.into_boxed_str());
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

    #[test]
    fn test_serialize_deserialize_value() {
        let op = Op::from(42.0);
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: Op<f32> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op.name(), deserialized.name());
        assert_eq!(op.arity(), deserialized.arity());
    }
}
