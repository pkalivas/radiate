pub mod bool;
pub mod expr;
pub mod math;
pub mod mutator;
pub mod operation;
mod param;
pub mod primitives;
#[cfg(feature = "serde")]
mod serde;

pub use expr::Expression;
pub use math::{activation_ops, all_ops, math_ops};
pub use mutator::OperationMutator;
pub use operation::*;
pub use param::Param;

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
    pub const LOGSUMEXP: &str = "logsumexp";

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
