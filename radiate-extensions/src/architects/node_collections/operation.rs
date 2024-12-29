use std::{
    fmt::Display,
    ops::{Add, Deref, Div, Mul, Neg, Sub},
    sync::Arc,
};

use rand::{
    distributions::{uniform::SampleUniform, Standard},
    prelude::Distribution,
};

use num_traits::{Float, NumCast};
use radiate::random_provider;

const MAX_VALUE: f32 = 1e+5_f32;
const MIN_VALUE: f32 = -1e+5_f32;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Arity {
    Zero,
    Nary(u8),
    Any,
}

impl From<u8> for Arity {
    fn from(value: u8) -> Self {
        match value {
            0 => Arity::Zero,
            n => Arity::Nary(n),
        }
    }
}

impl Deref for Arity {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        match self {
            Arity::Zero => &0,
            Arity::Nary(n) => n,
            Arity::Any => &0,
        }
    }
}

/// A generic operation type that can represent several kinds of “ops”.
pub enum Operation<T> {
    /// 1) A stateless function operation:
    ///    - A `&'static str` name (e.g., "Add", "Sigmoid")
    ///    - Arity (how many inputs it takes)
    ///    - Arc<dyn Fn(&[T]) -> T> for the actual function logic
    Fn(&'static str, Arity, Arc<dyn Fn(&[T]) -> T>),
    /// 2) A variable-like operation:
    ///    - `String` = a name or identifier
    ///    - `usize` = perhaps an index to retrieve from some external context
    Var(&'static str, usize),
    /// 3) A compile-time constant:
    ///    - `&'static str` name
    ///    - `T` the actual constant value
    Const(&'static str, T),
    /// 4) A `mutable const` is a constant that can change over time:
    ///  # Arguments
    /// - `&'static str` name
    /// - `Arity` of how many inputs it might read
    /// - Current value of type `T`
    /// - An `Arc<dyn Fn() -> T>` for retrieving (or resetting) the value
    /// - An `Arc<dyn Fn(&[T], &T) -> T>` for updating or combining inputs & old value -> new value
    ///
    ///    This suggests a node that can mutate its internal state over time, or
    ///    one that needs a special function to incorporate the inputs into the next state.
    MutableConst {
        name: &'static str,
        arity: Arity,
        value: T,
        get_value: Arc<dyn Fn() -> T>,
        operation: Arc<dyn Fn(&[T], &T) -> T>,
    },
}

unsafe impl Send for Operation<f32> {}
unsafe impl Sync for Operation<f32> {}

impl<T> Operation<T> {
    pub fn name(&self) -> &str {
        match self {
            Operation::Fn(name, _, _) => name,
            Operation::Var(name, _) => name,
            Operation::Const(name, _) => name,
            Operation::MutableConst { name, .. } => name,
        }
    }

    pub fn arity(&self) -> Arity {
        match self {
            Operation::Fn(_, arity, _) => *arity,
            Operation::Var(_, _) => Arity::Zero,
            Operation::Const(_, _) => Arity::Zero,
            Operation::MutableConst { arity, .. } => *arity,
        }
    }

    pub fn apply(&self, inputs: &[T]) -> T
    where
        T: Clone,
    {
        match self {
            Operation::Fn(_, _, op) => op(inputs),
            Operation::Var(_, index) => inputs[*index].clone(),
            Operation::Const(_, value) => value.clone(),
            Operation::MutableConst {
                value, operation, ..
            } => operation(inputs, value),
        }
    }

    pub fn new_instance(&self) -> Operation<T>
    where
        T: Clone,
    {
        match self {
            Operation::Fn(name, arity, op) => Operation::Fn(name, *arity, op.clone()),
            Operation::Var(name, index) => Operation::Var(name.clone(), *index),
            Operation::Const(name, value) => Operation::Const(name, value.clone()),
            Operation::MutableConst {
                name,
                arity,
                value: _,
                get_value,
                operation,
            } => Operation::MutableConst {
                name,
                arity: *arity,
                value: (*get_value)(),
                get_value: get_value.clone(),
                operation: operation.clone(),
            },
        }
    }
}

impl<T> Clone for Operation<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Operation::Fn(name, arity, op) => Operation::Fn(name, *arity, op.clone()),
            Operation::Var(name, index) => Operation::Var(name.clone(), *index),
            Operation::Const(name, value) => Operation::Const(name, value.clone()),
            Operation::MutableConst {
                name,
                arity,
                value,
                get_value,
                operation,
            } => Operation::MutableConst {
                name,
                arity: *arity,
                value: value.clone(),
                get_value: get_value.clone(),
                operation: operation.clone(),
            },
        }
    }
}

impl<T> PartialEq for Operation<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<T> std::fmt::Display for Operation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<T> Default for Operation<T>
where
    T: Default,
{
    fn default() -> Self {
        Operation::Const("default", T::default())
    }
}

impl<T> std::fmt::Debug for Operation<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Fn(name, _, _) => write!(f, "Fn: {}", name),
            Operation::Var(name, index) => write!(f, "Var: {}({})", name, index),
            Operation::Const(name, value) => write!(f, "C: {}({:?})", name, value),
            Operation::MutableConst { name, value, .. } => write!(f, "{}({:.2?})", name, value),
        }
    }
}

pub fn clamp<T>(value: T) -> T
where
    T: Clone + Float,
{
    if value.is_nan() {
        return T::from(0_f32).unwrap();
    }

    let min_value = T::from(MIN_VALUE).unwrap();
    let max_value = T::from(MAX_VALUE).unwrap();

    if value < min_value {
        min_value
    } else if value > max_value {
        max_value
    } else {
        value
    }
}

pub fn value<T: Clone + Display>(value: T) -> Operation<T> {
    let name = Box::leak(Box::new(format!("{}", value)));
    Operation::Const(name, value)
}

pub fn constant<T: Clone>(name: &'static str, value: T) -> Operation<T> {
    Operation::Const(name, value)
}

pub fn add<T: Add<Output = T> + Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "+",
        2.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0] + inputs[1])),
    )
}

pub fn sub<T: Sub<Output = T> + Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "-",
        2.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0] - inputs[1])),
    )
}

pub fn mul<T: Mul<Output = T> + Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "*",
        2.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0] * inputs[1])),
    )
}

pub fn div<T: Div<Output = T> + Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "/",
        2.into(),
        Arc::new(|inputs: &[T]| {
            let denom = if inputs[1] == T::from(0).unwrap() {
                inputs[0] / T::from(1).unwrap()
            } else {
                inputs[0] / inputs[1]
            };

            clamp(denom)
        }),
    )
}

pub fn sum<T: Add<Output = T> + Clone + Default + Float>() -> Operation<T> {
    Operation::Fn(
        "sum",
        Arity::Any,
        Arc::new(|inputs: &[T]| clamp(inputs.iter().fold(T::default(), |acc, x| acc + *x))),
    )
}

pub fn prod<T: Mul<Output = T> + Clone + Default + Float>() -> Operation<T> {
    Operation::Fn(
        "prod",
        Arity::Any,
        Arc::new(|inputs: &[T]| {
            let result = inputs.iter().fold(T::default(), |acc, x| acc * *x);

            clamp(result)
        }),
    )
}

pub fn neg<T: Neg<Output = T> + Clone + Default + Float>() -> Operation<T> {
    Operation::Fn("neg", 1.into(), Arc::new(|inputs: &[T]| clamp(-inputs[0])))
}

pub fn pow<T: Mul<Output = T> + Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "pow",
        2.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0] * inputs[1])),
    )
}

pub fn sqrt<T: Mul<Output = T> + Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "sqrt",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].sqrt())),
    )
}

pub fn abs<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "abs",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].abs())),
    )
}

pub fn exp<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "exp",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].exp())),
    )
}

pub fn log<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "log",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].ln())),
    )
}

pub fn sin<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "sin",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].sin())),
    )
}

pub fn cos<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "cos",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].cos())),
    )
}

pub fn tan<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "tan",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].tan())),
    )
}

pub fn ceil<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "ceil",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].ceil())),
    )
}

pub fn floor<T: Clone + Float>() -> Operation<T> {
    Operation::Fn(
        "floor",
        1.into(),
        Arc::new(|inputs: &[T]| clamp(inputs[0].floor())),
    )
}

pub fn gt<T: Clone + PartialEq + PartialOrd>() -> Operation<T> {
    Operation::Fn(
        ">",
        2.into(),
        Arc::new(|inputs: &[T]| {
            if inputs[0] > inputs[1] {
                inputs[0].clone()
            } else {
                inputs[1].clone()
            }
        }),
    )
}

pub fn lt<T: Clone + PartialEq + PartialOrd>() -> Operation<T> {
    Operation::Fn(
        "<",
        2.into(),
        Arc::new(|inputs: &[T]| {
            if inputs[0] < inputs[1] {
                inputs[0].clone()
            } else {
                inputs[1].clone()
            }
        }),
    )
}

pub fn max<T: Clone + PartialOrd>() -> Operation<T> {
    Operation::Fn(
        "max",
        Arity::Any,
        Arc::new(|inputs: &[T]| {
            inputs.iter().fold(
                inputs[0].clone(),
                |acc, x| {
                    if *x > acc {
                        x.clone()
                    } else {
                        acc
                    }
                },
            )
        }),
    )
}

pub fn min<T: Clone + PartialOrd>() -> Operation<T> {
    Operation::Fn(
        "min",
        Arity::Any,
        Arc::new(|inputs: &[T]| {
            inputs.iter().fold(
                inputs[0].clone(),
                |acc, x| {
                    if *x < acc {
                        x.clone()
                    } else {
                        acc
                    }
                },
            )
        }),
    )
}

pub fn weight<T: Sub<Output = T> + Mul<Output = T> + Copy + Default + Float>() -> Operation<T>
where
    Standard: Distribution<T>,
    T: PartialOrd + NumCast + SampleUniform,
{
    let supplier = || random_provider::random::<T>() * T::from(2).unwrap() - T::from(1).unwrap();
    let operation = |inputs: &[T], weight: &T| clamp(inputs[0] * *weight);
    Operation::MutableConst {
        name: "w",
        arity: 1.into(),
        value: supplier(),
        get_value: Arc::new(supplier),
        operation: Arc::new(operation),
    }
}

pub fn var<T: Clone>(index: usize) -> Operation<T> {
    let var_name = Box::leak(Box::new(format!("var_{}", index)));
    Operation::Var(var_name, index)
}

pub fn sigmoid() -> Operation<f32> {
    Operation::Fn(
        "sigmoid",
        Arity::Any,
        Arc::new(|inputs: &[f32]| {
            let sum = inputs.iter().fold(0_f32, |acc, x| acc + x);
            let result = 1_f32 / (1_f32 + (-sum).exp());
            clamp(result)
        }),
    )
}

pub fn relu() -> Operation<f32> {
    Operation::Fn(
        "relu",
        Arity::Any,
        Arc::new(|inputs: &[f32]| {
            let sum = inputs.iter().fold(0_f32, |acc, x| acc + x);
            let result = clamp(sum);
            if result > 0_f32 {
                result
            } else {
                0_f32
            }
        }),
    )
}

pub fn tanh() -> Operation<f32> {
    Operation::Fn(
        "tanh",
        Arity::Any,
        Arc::new(|inputs: &[f32]| {
            let result = inputs.iter().fold(0_f32, |acc, x| acc + x).tanh();

            clamp(result)
        }),
    )
}

pub fn linear() -> Operation<f32> {
    Operation::Fn(
        "linear",
        Arity::Any,
        Arc::new(|inputs: &[f32]| {
            let result = inputs.iter().fold(0_f32, |acc, x| acc + x);

            clamp(result)
        }),
    )
}

pub fn mish() -> Operation<f32> {
    Operation::Fn(
        "mish",
        Arity::Any,
        Arc::new(|inputs: &[f32]| {
            let result = inputs.iter().fold(0_f32, |acc, x| acc + x).tanh()
                * (inputs
                    .iter()
                    .fold(0_f32, |acc, x| acc + x)
                    .exp()
                    .ln_1p()
                    .exp());

            clamp(result)
        }),
    )
}

pub fn leaky_relu() -> Operation<f32> {
    Operation::Fn(
        "l_relu",
        Arity::Any,
        Arc::new(|inputs: &[f32]| {
            let sum = inputs.iter().fold(0_f32, |acc, x| acc + x);
            let result = if sum > 0_f32 { sum } else { 0.01 * sum };

            clamp(result)
        }),
    )
}

pub fn softplus() -> Operation<f32> {
    Operation::Fn(
        "soft_plus",
        Arity::Any,
        Arc::new(|inputs: &[f32]| {
            let sum = inputs.iter().fold(0_f32, |acc, x| acc + x);
            let result = sum.exp().ln_1p();

            clamp(result)
        }),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ops() {
        let op = add();
        assert_eq!(op.name(), "+");
        assert_eq!(op.arity(), Arity::Nary(2));
        assert_eq!(op.apply(&[1_f32, 2_f32]), 3_f32);
        assert_eq!(op.new_instance(), op);
    }

    #[test]
    fn test_random_seed_works() {
        random_provider::set_seed(42);

        let op = weight::<f32>();
        let op2 = weight::<f32>();

        let o_one = match op {
            Operation::MutableConst { value, .. } => value,
            _ => panic!("Expected MutableConst"),
        };

        let o_two = match op2 {
            Operation::MutableConst { value, .. } => value,
            _ => panic!("Expected MutableConst"),
        };

        println!("o_one: {:?}", o_one);
        println!("o_two: {:?}", o_two);
    }
}
