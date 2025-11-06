use crate::{Arity, Eval, Factory, NodeValue, TreeNode};
#[cfg(feature = "pgm")]
use std::sync::Arc;
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
        supplier: fn() -> T,
        modifier: fn(&T) -> T,
        operation: fn(&[T], &T) -> T,
    },
    /// 5) A Probabilistic Graph Model (PGM) operation that can be used to create complex functions that can
    /// be used to discover _how_ the inputs relate to the output and can be used to sample new inputs
    /// based on the learned relationships.
    ///
    /// # Arguments
    /// - `&'static str` name
    /// - `Arity` of how many inputs it might read
    /// - A `Vec<TreeNode<Op<T>>>` that can be used to learn from the inputs and generate new outputs based on the learned relationships.
    /// - An `Arc<dyn Fn(&[T], &[TreeNode<Op<T>>]) -> T>` for the actual function logic that uses the inputs and the PGM to produce an output.
    #[cfg(feature = "pgm")]
    PGM(
        &'static str,
        Arity,
        Arc<Vec<TreeNode<Op<T>>>>,
        fn(&[T], &[TreeNode<Op<T>>]) -> T,
    ),
}

impl<T> Op<T> {
    pub fn name(&self) -> &str {
        match self {
            Op::Fn(name, _, _) => name,
            Op::Var(name, _) => name,
            Op::Const(name, _) => name,
            Op::MutableConst { name, .. } => name,
            #[cfg(feature = "pgm")]
            Op::PGM(name, _, _, _) => name,
        }
    }

    pub fn arity(&self) -> Arity {
        match self {
            Op::Fn(_, arity, _) => *arity,
            Op::Var(_, _) => Arity::Zero,
            Op::Const(_, _) => Arity::Zero,
            Op::MutableConst { arity, .. } => *arity,
            #[cfg(feature = "pgm")]
            Op::PGM(_, arity, _, _) => *arity,
        }
    }

    pub fn is_fn(&self) -> bool {
        matches!(self, Op::Fn(_, _, _))
    }

    pub fn is_var(&self) -> bool {
        matches!(self, Op::Var(_, _))
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Op::Const(_, _))
    }

    pub fn is_mutable_const(&self) -> bool {
        matches!(self, Op::MutableConst { .. })
    }

    #[cfg(feature = "pgm")]
    pub fn is_pgm(&self) -> bool {
        matches!(self, Op::PGM(_, _, _, _))
    }
}

unsafe impl<T> Send for Op<T> {}
unsafe impl<T> Sync for Op<T> {}

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
            #[cfg(feature = "pgm")]
            Op::PGM(_, _, model, operation) => operation(inputs, &model),
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
                supplier: *supplier,
                modifier: *modifier,
                operation: *operation,
            },
            #[cfg(feature = "pgm")]
            Op::PGM(name, arity, model, operation) => {
                use std::sync::Arc;
                Op::PGM(name, *arity, Arc::clone(model), *operation)
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
                supplier: *supplier,
                modifier: *modifier,
                operation: *operation,
            },
            #[cfg(feature = "pgm")]
            Op::PGM(name, arity, model, operation) => {
                use std::sync::Arc;
                Op::PGM(name, *arity, Arc::clone(model), *operation)
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
            Op::Var(name, index) => write!(f, "Var: {}({})", name, index),
            Op::Const(name, value) => write!(f, "C: {}({:?})", name, value),
            Op::MutableConst { name, value, .. } => write!(f, "{}({:.2?})", name, value),
            #[cfg(feature = "pgm")]
            Op::PGM(name, _, model, _) => {
                let mut model_str = String::new();
                for (i, node) in model.iter().enumerate() {
                    use crate::Format;

                    let node_str = &node.format();
                    model_str.push_str(&format!("[{}: S {} Prog {}], ", i, node.size(), node_str));
                }
                write!(f, "PGM: {}({})", name, model_str)
            }
        }
    }
}

impl<T> From<Op<T>> for NodeValue<Op<T>> {
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

    #[test]
    #[cfg(feature = "pgm")]
    fn test_pgm_op() {
        use std::sync::Arc;
        let model = TreeNode::with_children(
            Op::add(),
            vec![
                TreeNode::new(Op::constant(1_f32)),
                TreeNode::new(Op::constant(2_f32)),
            ],
        );

        let pgm_op = Op::PGM(
            "pgm",
            Arity::Any,
            Arc::new(vec![model]),
            |inputs: &[f32], prog: &[TreeNode<Op<f32>>]| {
                let sum: f32 = prog.iter().map(|node| node.eval(inputs)).sum();
                sum + inputs.iter().sum::<f32>()
            },
        );

        let result = pgm_op.eval(&[]);
        assert_eq!(result, 3_f32);
    }
}
