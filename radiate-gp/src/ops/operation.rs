use std::{
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

use crate::{Eval, Factory, NodeCell};

/// Arity is a way to describe how many inputs an operation expects.
/// It can be zero, a specific number, or any number.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Arity {
    Zero,
    Exact(usize),
    Any,
}

impl From<usize> for Arity {
    fn from(value: usize) -> Self {
        match value {
            0 => Arity::Zero,
            n => Arity::Exact(n),
        }
    }
}

impl Deref for Arity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            Arity::Zero => &0,
            Arity::Exact(n) => n,
            Arity::Any => &0,
        }
    }
}

/// A generic operation type that can represent several kinds of “ops”.
pub enum Op<T> {
    /// 1) A stateless function operation:
    ///
    /// # Arguments
    ///    - A `&'static str` name (e.g., "Add", "Sigmoid")
    ///    - Arity (how many inputs it takes)
    ///    - Arc<dyn Fn(&[T]) -> T> for the actual function logic
    Fn(&'static str, Arity, Arc<dyn Fn(&[T]) -> T>),
    /// 2) A variable-like operation:
    ///
    /// # Arguments
    ///    - `String` = a name or identifier
    ///    - `usize` = perhaps an index to retrieve from some external context
    Var(&'static str, usize),
    /// 3) A compile-time constant: e.g., 1, 2, 3, etc.
    ///
    /// # Arguments
    ///    - `&'static str` name
    ///    - `T` the actual constant value
    Const(&'static str, T),
    /// 4) A `mutable const` is a constant that can change over time:
    ///
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
        modifier: Arc<dyn Fn(&T) -> T>,
        operation: Arc<dyn Fn(&[T], &T) -> T>,
    },
}

/// Base functionality for operations.
impl<T> Op<T> {
    pub fn name(&self) -> &str {
        match self {
            Op::Fn(name, _, _) => name,
            Op::Var(name, _) => name,
            Op::Const(name, _) => name,
            Op::MutableConst { name, .. } => name,
        }
    }

    pub fn arity(&self) -> Arity {
        match self {
            Op::Fn(_, arity, _) => *arity,
            Op::Var(_, _) => Arity::Zero,
            Op::Const(_, _) => Arity::Zero,
            Op::MutableConst { arity, .. } => *arity,
        }
    }

    pub fn value(value: T) -> Self
    where
        T: Clone + Display,
    {
        let name = Box::leak(Box::new(format!("{}", value)));
        Op::Const(name, value)
    }

    pub fn constant(name: &'static str, value: T) -> Self {
        Op::Const(name, value)
    }

    pub fn gt() -> Self
    where
        T: Clone + PartialEq + PartialOrd,
    {
        Op::Fn(
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

    pub fn lt() -> Self
    where
        T: Clone + PartialEq + PartialOrd,
    {
        Op::Fn(
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

    pub fn identity() -> Self
    where
        T: Clone,
    {
        Op::Fn(
            "Identity",
            1.into(),
            Arc::new(|inputs: &[T]| inputs[0].clone()),
        )
    }

    pub fn var(index: usize) -> Self
    where
        T: Clone,
    {
        let name = Box::leak(Box::new(format!("var_{}", index)));
        Op::Var(name, index)
    }
}

unsafe impl Send for Op<f32> {}
unsafe impl Sync for Op<f32> {}

impl<T> Eval<[T], T> for Op<T>
where
    T: Clone,
{
    fn eval(&self, inputs: &[T]) -> T {
        match self {
            Op::Fn(_, _, op) => op(inputs),
            Op::Var(_, index) => inputs[*index].clone(),
            Op::Const(_, value) => value.clone(),
            Op::MutableConst {
                value, operation, ..
            } => operation(inputs, value),
        }
    }
}

impl<T> Factory<Op<T>> for Op<T>
where
    T: Clone,
{
    type Input = ();

    fn new_instance(&self, _: Self::Input) -> Op<T> {
        match self {
            Op::Fn(name, arity, op) => Op::Fn(name, *arity, Arc::clone(op)),
            Op::Var(name, index) => Op::Var(name, *index),
            Op::Const(name, value) => Op::Const(name, value.clone()),
            Op::MutableConst {
                name,
                arity,
                value: _,
                get_value,
                modifier,
                operation,
            } => Op::MutableConst {
                name,
                arity: *arity,
                value: (*get_value)(),
                get_value: Arc::clone(get_value),
                modifier: Arc::clone(modifier),
                operation: Arc::clone(operation),
            },
        }
    }
}

impl<T: Clone> NodeCell for Op<T> {}

impl<T> Clone for Op<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Op::Fn(name, arity, op) => Op::Fn(name, *arity, Arc::clone(op)),
            Op::Var(name, index) => Op::Var(name, *index),
            Op::Const(name, value) => Op::Const(name, value.clone()),
            Op::MutableConst {
                name,
                arity,
                value,
                get_value,
                modifier,
                operation,
            } => Op::MutableConst {
                name,
                arity: *arity,
                value: value.clone(),
                get_value: Arc::clone(get_value),
                modifier: Arc::clone(modifier),
                operation: Arc::clone(operation),
            },
        }
    }
}

impl<T> PartialEq for Op<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl<T> Display for Op<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<T> Default for Op<T>
where
    T: Default,
{
    fn default() -> Self {
        Op::Const("default", T::default())
    }
}

impl<T> Debug for Op<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Fn(name, _, _) => write!(f, "Fn: {}", name),
            Op::Var(name, index) => write!(f, "Var: {}({})", name, index),
            Op::Const(name, value) => write!(f, "C: {}({:?})", name, value),
            Op::MutableConst { name, value, .. } => write!(f, "{}({:.2?})", name, value),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use radiate::random_provider;

    #[test]
    fn test_ops() {
        let op = Op::add();
        assert_eq!(op.name(), "add");
        assert_eq!(op.arity(), Arity::Exact(2));
        assert_eq!(op.eval(&vec![1_f32, 2_f32]), 3_f32);
        assert_eq!(op.new_instance(()), op);
    }

    #[test]
    fn test_random_seed_works() {
        random_provider::set_seed(42);

        let op = Op::weight();
        let op2 = Op::weight();

        let o_one = match op {
            Op::MutableConst { value, .. } => value,
            _ => panic!("Expected MutableConst"),
        };

        let o_two = match op2 {
            Op::MutableConst { value, .. } => value,
            _ => panic!("Expected MutableConst"),
        };

        println!("o_one: {:?}", o_one);
        println!("o_two: {:?}", o_two);
    }
}
