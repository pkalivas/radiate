use super::Op;
use crate::ops::op_names;

impl Op<bool> {
    pub fn and() -> Self {
        Op::Fn(op_names::AND, 2.into(), |args: &[bool]| args[0] && args[1])
    }

    pub fn or() -> Self {
        Op::Fn(op_names::OR, 2.into(), |args: &[bool]| args[0] || args[1])
    }

    pub fn not() -> Self {
        Op::Fn(op_names::NOT, 1.into(), |args: &[bool]| !args[0])
    }

    pub fn xor() -> Self {
        Op::Fn(op_names::XOR, 2.into(), |args: &[bool]| args[0] ^ args[1])
    }

    pub fn eq() -> Self {
        Op::Fn(op_names::EQ, 2.into(), |args: &[bool]| args[0] == args[1])
    }

    pub fn ne() -> Self {
        Op::Fn(op_names::NE, 2.into(), |args: &[bool]| args[0] != args[1])
    }

    pub fn gt() -> Self {
        Op::Fn(op_names::GT, 2.into(), |args: &[bool]| args[0] & !args[1])
    }

    pub fn ge() -> Self {
        Op::Fn(op_names::GE, 2.into(), |args: &[bool]| args[0] >= args[1])
    }

    pub fn lt() -> Self {
        Op::Fn(op_names::LT, 2.into(), |args: &[bool]| !args[0] & args[1])
    }

    pub fn le() -> Self {
        Op::Fn(op_names::LE, 2.into(), |args: &[bool]| args[0] <= args[1])
    }

    pub fn if_else() -> Self {
        Op::Fn(op_names::IF_ELSE, 3.into(), |args: &[bool]| {
            if args[0] { args[1] } else { args[2] }
        })
    }

    pub fn and_then() -> Self {
        Op::Fn(op_names::AND_THEN, 3.into(), |args: &[bool]| {
            args[0] && args[1] && args[2]
        })
    }

    pub fn or_else() -> Self {
        Op::Fn(op_names::OR_ELSE, 2.into(), |args: &[bool]| {
            args[0] || args[1] || args[2]
        })
    }

    pub fn nand() -> Self {
        Op::Fn(op_names::NAND, 2.into(), |args: &[bool]| {
            !(args[0] && args[1])
        })
    }

    pub fn nor() -> Self {
        Op::Fn(op_names::NOR, 2.into(), |args: &[bool]| {
            !(args[0] || args[1])
        })
    }

    pub fn xnor() -> Self {
        Op::Fn(op_names::XNOR, 2.into(), |args: &[bool]| {
            !(args[0] ^ args[1])
        })
    }

    pub fn implies() -> Self {
        Op::Fn(op_names::IMPLIES, 2.into(), |args: &[bool]| {
            !args[0] || args[1]
        })
    }

    pub fn iff() -> Self {
        Op::Fn(op_names::IFF, 2.into(), |args: &[bool]| args[0] == args[1])
    }
}
