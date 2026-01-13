use crate::{Arity, Eval, Factory, NodeValue, TreeNode, ops::OpValue};
use radiate_utils::Value;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// [Op] is an enumeration that represents the different types of operations
/// that can be performed within the genetic programming framework. Each variant
/// of the enum encapsulates a different kind of operation, allowing for a flexible
/// and extensible way to define the behavior of nodes within trees and graphs.
///
/// The [Op] heavilty depends on it's [Arity] to define how many inputs it expects.
/// This is crucial for ensuring that the operations receive the correct number of inputs
/// and that the structures built using these operations are built in ways that respect
/// these input requirements. For example, an addition operation would typically have an arity of 2,
/// while a constant operation would have an arity of 0. This is the _base_ level of the GP system, meaning
/// that everything built on top of it (trees, graphs, etc.) will relies *heavily* on how these
/// operations are defined and used.
pub enum Op<T> {
    /// 1) A stateless function operation:
    ///
    /// # Arguments
    ///    - A `&'static str` name (e.g., "Add", "Sigmoid")
    ///    - Arity (how many inputs it takes)
    ///    - Arc<dyn Fn(&`\[`T`\]`) -> T> for the actual function logic
    Fn(&'static str, Arity, fn(&[T]) -> T),
    /// 2) A variable-like operation:
    ///
    /// # Arguments
    ///    - `String` = a name or identifier
    ///    - `usize` = an index to retrieve from some external context
    ///   - `Option<usize>` = an optional domain size for categorical variables
    Var(&'static str, usize, Option<usize>),
    /// 3) A compile-time constant: e.g., 1, 2, 3, etc.
    ///
    /// # Arguments
    ///    - `&'static str` name
    ///    - `T` the actual constant value
    Const(&'static str, T),
    /// 4) A value-based operation that encapsulates data and an operation to process it.
    ///
    /// This allows for operations that can hold state or data, such as weights in a neural
    /// network, and apply a specific function to that data when evaluated.
    ///
    /// # Arguments
    /// - `&'static str` name
    /// - `Arity` of how many inputs it might read
    /// - `OpValue<T>` the actual data/value associated with this operation
    /// - An `fn(&[T], &Value<T>) -> T` for the function logic that uses the inputs and the value to produce an output.
    Value(&'static str, Arity, OpValue<T>, fn(&[T], &Value<T>) -> T),
}

impl<T> Op<T> {
    pub fn name(&self) -> &str {
        match self {
            Op::Fn(name, _, _) => name,
            Op::Var(name, _, _) => name,
            Op::Const(name, _) => name,
            Op::Value(name, _, _, _) => name,
        }
    }

    pub fn arity(&self) -> Arity {
        match self {
            Op::Fn(_, arity, _) => *arity,
            Op::Var(_, _, _) => Arity::Zero,
            Op::Const(_, _) => Arity::Zero,
            Op::Value(_, arity, _, _) => *arity,
        }
    }

    pub fn value(&self) -> Option<&Value<T>> {
        match self {
            Op::Value(_, _, value, _) => Some(value.data()),
            _ => None,
        }
    }

    pub fn domain(&self) -> Option<&usize> {
        match self {
            Op::Var(_, _, card) => card.as_ref(),
            _ => None,
        }
    }

    pub fn is_fn(&self) -> bool {
        matches!(self, Op::Fn(_, _, _))
    }

    pub fn is_var(&self) -> bool {
        matches!(self, Op::Var(_, _, _))
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Op::Const(_, _))
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Op::Value(_, _, _, _))
    }
}

impl<T> Eval<[T], T> for Op<T>
where
    T: Clone,
{
    fn eval(&self, inputs: &[T]) -> T {
        match self {
            Op::Fn(_, _, op) => op(inputs),
            Op::Var(_, index, _) => inputs[*index].clone(),
            Op::Const(_, value) => value.clone(),
            Op::Value(_, _, value, operation) => operation(inputs, value.data()),
        }
    }
}

impl<T> Factory<(), Op<T>> for Op<T>
where
    T: Clone,
{
    fn new_instance(&self, _: ()) -> Op<T> {
        match self {
            Op::Fn(name, arity, op) => Op::Fn(name, *arity, *op),
            Op::Var(name, index, domain) => Op::Var(name, *index, domain.clone()),
            Op::Const(name, value) => Op::Const(name, value.clone()),
            Op::Value(name, arity, value, operation) => {
                Op::Value(name, *arity, value.new_instance(()), *operation)
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
            Op::Fn(name, arity, op) => Op::Fn(name, *arity, *op),
            Op::Var(name, index, domain) => Op::Var(name, *index, domain.clone()),
            Op::Const(name, value) => Op::Const(name, value.clone()),
            Op::Value(name, arity, value, operation) => {
                Op::Value(name, *arity, value.clone(), *operation)
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
            && self.arity() == other.arity()
            && match (self, other) {
                (Op::Fn(_, _, _), Op::Fn(_, _, _)) => true,
                (Op::Var(_, idx_a, card_a), Op::Var(_, idx_b, card_b)) => {
                    idx_a == idx_b && card_a == card_b
                }
                (Op::Const(_, val_a), Op::Const(_, val_b)) => val_a == val_b,
                (Op::Value(_, _, val_a, _), Op::Value(_, _, val_b, _)) => val_a == val_b,
                _ => false,
            }
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
        Op::Fn("default", Arity::Zero, |_: &[T]| T::default())
    }
}

impl<T> Debug for Op<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Fn(name, _, _) => write!(f, "Fn: {}", name),
            Op::Var(name, index, card) => match card {
                Some(k) => {
                    write!(f, "Var: {}({}, Cat:{})", name, index, k)
                }
                None => write!(f, "Var: {}({})", name, index),
            },
            Op::Const(name, value) => write!(f, "C: {}({:?})", name, value),
            Op::Value(name, _, value, _) => {
                write!(f, "Val: {}({:?})", name, value)
            }
        }
    }
}

impl<T: Clone> From<Op<T>> for NodeValue<Op<T>> {
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

impl<T> From<Op<T>> for Vec<TreeNode<Op<T>>> {
    fn from(value: Op<T>) -> Self {
        vec![TreeNode::from(value)]
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
            Op::Value(_, _, value, _) => value,
            _ => panic!("Expected Value"),
        };

        let o_two = match op2 {
            Op::Value(_, _, value, _) => value,
            _ => panic!("Expected Value"),
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
