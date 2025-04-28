use crate::{Arity, Eval, Factory, NodeValue, TreeNode};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    sync::Arc,
};

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
        supplier: Arc<dyn Fn() -> T>,
        modifier: Arc<dyn Fn(&T) -> T>,
        operation: Arc<dyn Fn(&[T], &T) -> T>,
    },
    /// 5) A 'Value' operation that can be used as a 'stateful' constant:
    ///
    /// # Arguments
    /// - `&'static str` name
    /// - `Arity` of how many inputs it might read
    /// - Current value of type `T`
    /// - An `Arc<dyn Fn(&[T], &T) -> T>` for updating or combining inputs & old value -> new value
    Value(&'static str, Arity, T, Arc<dyn Fn(&[T], &T) -> T>),
}

/// Base functionality for operations.
impl<T> Op<T> {
    pub fn name(&self) -> &str {
        match self {
            Op::Fn(name, _, _) => name,
            Op::Var(name, _) => name,
            Op::Const(name, _) => name,
            Op::MutableConst { name, .. } => name,
            Op::Value(name, _, _, _) => name,
        }
    }

    pub fn arity(&self) -> Arity {
        match self {
            Op::Fn(_, arity, _) => *arity,
            Op::Var(_, _) => Arity::Zero,
            Op::Const(_, _) => Arity::Zero,
            Op::MutableConst { arity, .. } => *arity,
            Op::Value(_, arity, _, _) => *arity,
        }
    }

    pub fn constant(value: T) -> Self
    where
        T: Display,
    {
        let name = Box::leak(Box::new(format!("{}", value)));
        Op::Const(name, value)
    }

    pub fn named_constant(name: &'static str, value: T) -> Self {
        Op::Const(name, value)
    }

    pub fn identity() -> Self
    where
        T: Clone,
    {
        Op::Fn(
            "identity",
            1.into(),
            Arc::new(|inputs: &[T]| inputs[0].clone()),
        )
    }

    pub fn var(index: usize) -> Self {
        let name = Box::leak(Box::new(format!("{}", index)));
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
            Op::Value(_, _, value, operation) => operation(inputs, value),
        }
    }
}

impl<T> Factory<(), Op<T>> for Op<T>
where
    T: Clone,
{
    fn new_instance(&self, _: ()) -> Op<T> {
        match self {
            Op::Fn(name, arity, op) => Op::Fn(name, *arity, Arc::clone(op)),
            Op::Var(name, index) => Op::Var(name, *index),
            Op::Const(name, value) => Op::Const(name, value.clone()),
            Op::MutableConst {
                name,
                arity,
                value: _,
                supplier,
                modifier,
                operation,
            } => Op::MutableConst {
                name,
                arity: *arity,
                value: (*supplier)(),
                supplier: Arc::clone(supplier),
                modifier: Arc::clone(modifier),
                operation: Arc::clone(operation),
            },
            Op::Value(name, arity, value, operation) => {
                Op::Value(name, *arity, value.clone(), Arc::clone(operation))
            }
        }
    }
}

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
                supplier,
                modifier,
                operation,
            } => Op::MutableConst {
                name,
                arity: *arity,
                value: value.clone(),
                supplier: Arc::clone(supplier),
                modifier: Arc::clone(modifier),
                operation: Arc::clone(operation),
            },
            Op::Value(name, arity, value, operation) => {
                Op::Value(name, *arity, value.clone(), Arc::clone(operation))
            }
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

impl<T> Hash for Op<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
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
        Op::Fn("default", Arity::Zero, Arc::new(|_| T::default()))
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
            Op::Value(name, _, value, _) => write!(f, "{}({:.2?})", name, value),
        }
    }
}

impl<T> From<Op<T>> for NodeValue<Op<T>>
where
    T: Clone,
{
    fn from(value: Op<T>) -> Self {
        let arity = value.arity();
        NodeValue::Bounded(value, arity)
    }
}

impl<T> From<Op<T>> for TreeNode<Op<T>> {
    fn from(value: Op<T>) -> Self {
        let arity = value.arity();
        TreeNode::with_arity(value, arity)
    }
}

impl From<f32> for Op<f32> {
    fn from(value: f32) -> Self {
        Op::Value("Value(f32)", Arity::Any, value, Arc::new(|_, v| *v))
    }
}

impl From<i32> for Op<i32> {
    fn from(value: i32) -> Self {
        Op::Value("Value(i32)", Arity::Any, value, Arc::new(|_, v| *v))
    }
}

impl From<bool> for Op<bool> {
    fn from(value: bool) -> Self {
        Op::Value("Value(bool)", Arity::Any, value, Arc::new(|_, v| *v))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use radiate_core::random_provider;

    #[test]
    fn test_ops() {
        let op = Op::add();
        assert_eq!(op.name(), "add");
        assert_eq!(op.arity(), Arity::Exact(2));
        assert_eq!(op.eval(&[1_f32, 2_f32]), 3_f32);
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

    #[test]
    fn test_op_clone() {
        let op = Op::add();
        let op2 = op.clone();

        let result = op.eval(&[1_f32, 2_f32]);
        let result2 = op2.eval(&[1_f32, 2_f32]);

        assert_eq!(op, op2);
        assert_eq!(result, result2);
    }
}
