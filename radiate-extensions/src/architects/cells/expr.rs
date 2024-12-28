use std::{
    ops::{Add, Div, Mul, Neg, Sub},
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

pub enum Expr<T> {
    Fn(&'static str, u8, Arc<dyn Fn(&[T]) -> T>),
    Value(T),
    Var(String, usize),
    Const(&'static str, T),
    MutableConst(
        &'static str,
        u8,
        T,
        Arc<dyn Fn() -> T>,
        Arc<dyn Fn(&[T], &T) -> T>,
    ),
}

unsafe impl Send for Expr<f32> {}
unsafe impl Sync for Expr<f32> {}

impl<T> Expr<T> {
    pub fn name(&self) -> &str {
        match self {
            Expr::Fn(name, _, _) => name,
            Expr::Value(_) => "value",
            Expr::Var(name, _) => name,
            Expr::Const(name, _) => name,
            Expr::MutableConst(name, _, _, _, _) => name,
        }
    }

    pub fn arity(&self) -> u8 {
        match self {
            Expr::Fn(_, arity, _) => *arity,
            Expr::Value(_) => 0,
            Expr::Var(_, _) => 0,
            Expr::Const(_, _) => 0,
            Expr::MutableConst(_, arity, _, _, _) => *arity,
        }
    }

    pub fn apply(&self, inputs: &[T]) -> T
    where
        T: Clone,
    {
        match self {
            Expr::Fn(_, _, op) => op(inputs),
            Expr::Value(value) => value.clone(),
            Expr::Var(_, index) => inputs[*index].clone(),
            Expr::Const(_, value) => value.clone(),
            Expr::MutableConst(_, _, value, _, operation) => operation(inputs, value),
        }
    }

    pub fn new_instance(&self) -> Expr<T>
    where
        T: Clone,
    {
        match self {
            Expr::Fn(name, arity, op) => Expr::Fn(name, *arity, op.clone()),
            Expr::Value(value) => Expr::Value(value.clone()),
            Expr::Var(name, index) => Expr::Var(name.clone(), *index),
            Expr::Const(name, value) => Expr::Const(name, value.clone()),
            Expr::MutableConst(name, arity, _, get_value, operation) => Expr::MutableConst(
                name,
                *arity,
                get_value().clone(),
                get_value.clone(),
                operation.clone(),
            ),
        }
    }
}

impl<T> Clone for Expr<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Expr::Fn(name, arity, op) => Expr::Fn(name, *arity, op.clone()),
            Expr::Value(value) => Expr::Value(value.clone()),
            Expr::Var(name, index) => Expr::Var(name.clone(), *index),
            Expr::Const(name, value) => Expr::Const(name, value.clone()),
            Expr::MutableConst(name, arity, value, get_value, operation) => Expr::MutableConst(
                name,
                *arity,
                value.clone(),
                get_value.clone(),
                operation.clone(),
            ),
        }
    }
}

impl<T> PartialEq for Expr<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Fn(name, arity, _), Expr::Fn(other_name, other_arity, _)) => {
                name == other_name && arity == other_arity
            }
            (Expr::Value(value), Expr::Value(other_value)) => value == other_value,
            (Expr::Var(name, index), Expr::Var(other_name, other_index)) => {
                name == other_name && index == other_index
            }
            (Expr::Const(name, value), Expr::Const(other_name, other_value)) => {
                name == other_name && value == other_value
            }
            (
                Expr::MutableConst(name, arity, value, _, _),
                Expr::MutableConst(other_name, other_arity, other_value, _, _),
            ) => name == other_name && arity == other_arity && value == other_value,
            _ => false,
        }
    }
}

impl<T> std::fmt::Display for Expr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<T> Default for Expr<T>
where
    T: Default,
{
    fn default() -> Self {
        Expr::Const("default", T::default())
    }
}

impl<T> std::fmt::Debug for Expr<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Fn(name, _, _) => write!(f, "Fn: {}", name),
            Expr::Value(value) => write!(f, "Val: {:?}", value),
            Expr::Var(name, index) => write!(f, "Var: {}({})", name, index),
            Expr::Const(name, value) => write!(f, "C: {}({:?})", name, value),
            Expr::MutableConst(name, _, value, _, _) => write!(f, "{}({:.2?})", name, value),
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

pub fn value<T: Clone>(value: T) -> Expr<T> {
    Expr::Value(value)
}

pub fn add<T: Add<Output = T> + Clone + Float>() -> Expr<T> {
    Expr::Fn(
        "+",
        2,
        Arc::new(|inputs: &[T]| clamp(inputs[0] + inputs[1])),
    )
}

pub fn sub<T: Sub<Output = T> + Clone + Float>() -> Expr<T> {
    Expr::Fn(
        "-",
        2,
        Arc::new(|inputs: &[T]| clamp(inputs[0] - inputs[1])),
    )
}

pub fn mul<T: Mul<Output = T> + Clone + Float>() -> Expr<T> {
    Expr::Fn(
        "*",
        2,
        Arc::new(|inputs: &[T]| clamp(inputs[0] * inputs[1])),
    )
}

pub fn div<T: Div<Output = T> + Clone + Float>() -> Expr<T> {
    Expr::Fn(
        "/",
        2,
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

pub fn sum<T: Add<Output = T> + Clone + Default + Float>() -> Expr<T> {
    Expr::Fn(
        "sum",
        2,
        Arc::new(|inputs: &[T]| clamp(inputs.iter().fold(T::default(), |acc, x| acc + *x))),
    )
}

pub fn prod<T: Mul<Output = T> + Clone + Default + Float>() -> Expr<T> {
    Expr::Fn(
        "prod",
        2,
        Arc::new(|inputs: &[T]| {
            let result = inputs.iter().fold(T::default(), |acc, x| acc * *x);

            clamp(result)
        }),
    )
}

pub fn neg<T: Neg<Output = T> + Clone + Default + Float>() -> Expr<T> {
    Expr::Fn("neg", 1, Arc::new(|inputs: &[T]| clamp(-inputs[0])))
}

pub fn pow<T: Mul<Output = T> + Clone + Float>() -> Expr<T> {
    Expr::Fn(
        "pow",
        2,
        Arc::new(|inputs: &[T]| clamp(inputs[0] * inputs[1])),
    )
}

pub fn sqrt<T: Mul<Output = T> + Clone + Float>() -> Expr<T> {
    Expr::Fn("sqrt", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].sqrt())))
}

pub fn abs<T: Clone + Float>() -> Expr<T> {
    Expr::Fn("abs", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].abs())))
}

pub fn exp<T: Clone + Float>() -> Expr<T> {
    Expr::Fn("exp", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].exp())))
}

pub fn log<T: Clone + Float>() -> Expr<T> {
    Expr::Fn("log", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].ln())))
}

pub fn sin<T: Clone + Float>() -> Expr<T> {
    Expr::Fn("sin", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].sin())))
}

pub fn cos<T: Clone + Float>() -> Expr<T> {
    Expr::Fn("cos", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].cos())))
}

pub fn tan<T: Clone + Float>() -> Expr<T> {
    Expr::Fn("tan", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].tan())))
}

pub fn ceil<T: Clone + Float>() -> Expr<T> {
    Expr::Fn("ceil", 1, Arc::new(|inputs: &[T]| clamp(inputs[0].ceil())))
}

pub fn floor<T: Clone + Float>() -> Expr<T> {
    Expr::Fn(
        "floor",
        1,
        Arc::new(|inputs: &[T]| clamp(inputs[0].floor())),
    )
}

pub fn gt<T: Clone + PartialEq + PartialOrd>() -> Expr<T> {
    Expr::Fn(
        ">",
        2,
        Arc::new(|inputs: &[T]| {
            if inputs[0] > inputs[1] {
                inputs[0].clone()
            } else {
                inputs[1].clone()
            }
        }),
    )
}

pub fn lt<T: Clone + PartialEq + PartialOrd>() -> Expr<T> {
    Expr::Fn(
        "<",
        2,
        Arc::new(|inputs: &[T]| {
            if inputs[0] < inputs[1] {
                inputs[0].clone()
            } else {
                inputs[1].clone()
            }
        }),
    )
}

pub fn max<T: Clone + PartialOrd>() -> Expr<T> {
    Expr::Fn(
        "max",
        2,
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

pub fn min<T: Clone + PartialOrd>() -> Expr<T> {
    Expr::Fn(
        "min",
        2,
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

pub fn weight<T: Sub<Output = T> + Mul<Output = T> + Copy + Default + Float>() -> Expr<T>
where
    Standard: Distribution<T>,
    T: PartialOrd + NumCast + SampleUniform,
{
    let supplier = || random_provider::random::<T>() * T::from(2).unwrap() - T::from(1).unwrap();
    let operation = |inputs: &[T], weight: &T| clamp(inputs[0] * *weight);
    Expr::MutableConst("w", 1, supplier(), Arc::new(supplier), Arc::new(operation))
}

pub fn var<T: Clone>(index: usize) -> Expr<T> {
    let var_name = format!("x{}", index);
    Expr::Var(var_name, index)
}

pub fn sigmoid() -> Expr<f32> {
    Expr::Fn(
        "sigmoid",
        1,
        Arc::new(|inputs: &[f32]| {
            let sum = inputs.iter().fold(0_f32, |acc, x| acc + x);
            let result = 1_f32 / (1_f32 + (-sum).exp());
            clamp(result)
        }),
    )
}

pub fn relu() -> Expr<f32> {
    Expr::Fn(
        "relu",
        1,
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

pub fn tanh() -> Expr<f32> {
    Expr::Fn(
        "tanh",
        1,
        Arc::new(|inputs: &[f32]| {
            let result = inputs.iter().fold(0_f32, |acc, x| acc + x).tanh();

            clamp(result)
        }),
    )
}

pub fn linear() -> Expr<f32> {
    Expr::Fn(
        "linear",
        1,
        Arc::new(|inputs: &[f32]| {
            let result = inputs.iter().fold(0_f32, |acc, x| acc + x);

            clamp(result)
        }),
    )
}

pub fn mish() -> Expr<f32> {
    Expr::Fn(
        "mish",
        1,
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

pub fn leaky_relu() -> Expr<f32> {
    Expr::Fn(
        "l_relu",
        1,
        Arc::new(|inputs: &[f32]| {
            let sum = inputs.iter().fold(0_f32, |acc, x| acc + x);
            let result = if sum > 0_f32 { sum } else { 0.01 * sum };

            clamp(result)
        }),
    )
}

pub fn softplus() -> Expr<f32> {
    Expr::Fn(
        "soft_plus",
        1,
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
        assert_eq!(op.arity(), 2);
        assert_eq!(op.apply(&[1_f32, 2_f32]), 3_f32);
        assert_eq!(op.new_instance(), op);
    }

    #[test]
    fn test_random_seed_works() {
        random_provider::set_seed(42);

        let op = weight::<f32>();
        let op2 = weight::<f32>();

        let o_one = match op {
            Expr::MutableConst(_, _, value, _, _) => value,
            _ => panic!("Expected MutableConst"),
        };

        let o_two = match op2 {
            Expr::MutableConst(_, _, value, _, _) => value,
            _ => panic!("Expected MutableConst"),
        };

        println!("o_one: {:?}", o_one);
        println!("o_two: {:?}", o_two);
    }
}
