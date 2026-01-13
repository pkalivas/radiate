pub mod bool;
pub mod expr;
pub mod math;
pub mod mutator;
pub mod operation;
pub mod primitives;
pub mod prob;
#[cfg(feature = "serde")]
mod serde;
mod store;
mod value;

pub use expr::Expression;
pub use math::{activation_ops, all_ops, math_ops};
pub use mutator::OperationMutator;
pub use operation::*;
pub use prob::*;
pub use value::OpValue;

pub(crate) mod op_names {
    /// Mathematical operation names
    pub const ADD: &str = "add";
    pub const SUB: &str = "sub";
    pub const MUL: &str = "mul";
    pub const DIV: &str = "div";
    pub const SUM: &str = "sum";
    pub const DIFF: &str = "diff";
    pub const PROD: &str = "prod";
    pub const NEG: &str = "neg";
    pub const ABS: &str = "abs";
    pub const SQRT: &str = "sqrt";
    pub const POW: &str = "pow";
    pub const MAX: &str = "max";
    pub const MIN: &str = "min";
    pub const SIN: &str = "sin";
    pub const COS: &str = "cos";
    pub const TAN: &str = "tan";
    pub const EXP: &str = "exp";
    pub const LOG: &str = "log";
    pub const CEIL: &str = "ceil";
    pub const FLOOR: &str = "floor";
    pub const SOFTPLUS: &str = "softplus";
    pub const MISH: &str = "mish";
    pub const SWISH: &str = "swish";
    pub const ELU: &str = "elu";
    pub const LEAKY_RELU: &str = "l_relu";
    pub const LINEAR: &str = "linear";
    pub const RELU: &str = "relu";
    pub const SIGMOID: &str = "sigmoid";
    pub const TANH: &str = "tanh";
    pub const IDENTITY: &str = "identity";
    pub const WEIGHT: &str = "w";

    // Table operation names
    pub const ND_ARRAY: &str = "nd_array";
    pub const LOGPROB_TABLE: &str = "logprob_table";
    pub const GAUSS1: &str = "gauss_1d";
    pub const GAUSS_LIN2: &str = "gauss_lin_2d";

    /// Boolean operation names
    pub const AND: &str = "and";
    pub const OR: &str = "or";
    pub const NOT: &str = "not";
    pub const XOR: &str = "xor";
    pub const EQ: &str = "eq";
    pub const NE: &str = "ne";
    pub const GT: &str = "gt";
    pub const GE: &str = "ge";
    pub const LT: &str = "lt";
    pub const LE: &str = "le";
    pub const IF_ELSE: &str = "if_else";
    pub const AND_THEN: &str = "and_then";
    pub const OR_ELSE: &str = "or_else";
    pub const NAND: &str = "nand";
    pub const NOR: &str = "nor";
    pub const XNOR: &str = "xnor";
    pub const IMPLIES: &str = "implies";
    pub const IFF: &str = "iff";
}

// impl<T> Factory<(usize, &[usize]), GraphNode<Op<T>>> for NodeStore<Op<T>>
// where
//     T: Default + Clone,
// {
//     fn new_instance(&self, (index, scope): (usize, &[usize])) -> GraphNode<Op<T>> {
//         self.map_by_type(NodeType::Input, |values| {
//             let mapped_values = values
//                 .into_iter()
//                 .filter(|value| match value {
//                     NodeValue::Bounded(val, _) => match val {
//                         Op::Var(_, idx, domain) => scope.contains(idx),
//                         _ => false,
//                     },
//                     NodeValue::Unbound(val) => match val {
//                         Op::Var(_, idx, domain) => scope.contains(idx),
//                         _ => false,
//                     },
//                 })
//                 .collect::<Vec<&NodeValue<Op<T>>>>();

//             if mapped_values.is_empty() {
//                 self.new_instance((index, NodeType::Input))
//             } else {
//                 let node_value = random_provider::choose(&mapped_values);

//                 match node_value {
//                     NodeValue::Bounded(value, arity) => {
//                         GraphNode::with_arity(index, NodeType::Input, value.clone(), *arity)
//                     }
//                     NodeValue::Unbound(value) => {
//                         GraphNode::new(index, NodeType::Input, value.clone())
//                     }
//                 }
//             }
//         })
//         .unwrap_or(GraphNode::new(index, NodeType::Input, Op::default()))
//     }
// }

// use std::sync::Arc;

// #[derive(Hash)]
// struct OpValueInner<T> {
//     data: Value<T>,
//     supplier: fn(&Value<T>) -> Value<T>,
//     modifier: fn(&mut Value<T>),
// }

// #[derive(Clone)]
// pub struct OpValue<T> {
//     inner: Arc<OpValueInner<T>>,
// }

// impl<T> OpValue<T> {
//     pub fn new(
//         data: Value<T>,
//         supplier: fn(&Value<T>) -> Value<T>,
//         modifier: fn(&mut Value<T>),
//     ) -> Self {
//         Self {
//             inner: Arc::new(OpValueInner { data, supplier, modifier }),
//         }
//     }

//     #[inline]
//     pub fn data(&self) -> &Value<T> {
//         &self.inner.data
//     }

//     #[inline]
//     pub fn supplier(&self) -> fn(&Value<T>) -> Value<T> {
//         self.inner.supplier
//     }

//     #[inline]
//     pub fn modifier(&self) -> fn(&mut Value<T>) {
//         self.inner.modifier
//     }
// }

// impl<T: Clone> OpValue<T> {
//     /// Copy-on-write: if multiple ops share this value, they still point at the same inner
//     /// until you clone the OpValue handle and mutate via a different handle.
//     #[inline]
//     pub fn data_mut(&mut self) -> &mut Value<T> {
//         &mut Arc::make_mut(&mut self.inner).data
//     }
// }

// impl<T: Clone> Factory<(), OpValue<T>> for OpValue<T> {
//     fn new_instance(&self, _: ()) -> OpValue<T> {
//         let data = (self.inner.supplier)(&self.inner.data);
//         OpValue::new(data, self.inner.supplier, self.inner.modifier)
//     }
// }

// impl<T: Clone> Factory<Value<T>, OpValue<T>> for OpValue<T> {
//     fn new_instance(&self, mut val: Value<T>) -> OpValue<T> {
//         (self.inner.modifier)(&mut val);
//         OpValue::new(val, self.inner.supplier, self.inner.modifier)
//     }
// }
