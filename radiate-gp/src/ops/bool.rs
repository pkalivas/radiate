use super::Op;
use std::sync::Arc;

impl Op<bool> {
    pub fn and() -> Self {
        Op::Fn("and", 2.into(), Arc::new(|args| args[0] && args[1]))
    }

    pub fn or() -> Self {
        Op::Fn("or", 2.into(), Arc::new(|args| args[0] || args[1]))
    }

    pub fn not() -> Self {
        Op::Fn("not", 1.into(), Arc::new(|args| !args[0]))
    }

    pub fn xor() -> Self {
        Op::Fn("xor", 2.into(), Arc::new(|args| args[0] ^ args[1]))
    }

    pub fn eq() -> Self {
        Op::Fn("eq", 2.into(), Arc::new(|args| args[0] == args[1]))
    }

    pub fn ne() -> Self {
        Op::Fn("ne", 2.into(), Arc::new(|args| args[0] != args[1]))
    }

    pub fn gt() -> Self {
        Op::Fn("gt", 2.into(), Arc::new(|args| args[0] & !args[1]))
    }

    pub fn ge() -> Self {
        Op::Fn("ge", 2.into(), Arc::new(|args| args[0] >= args[1]))
    }

    pub fn lt() -> Self {
        Op::Fn("lt", 2.into(), Arc::new(|args| !args[0] & args[1]))
    }

    pub fn le() -> Self {
        Op::Fn("le", 2.into(), Arc::new(|args| args[0] <= args[1]))
    }

    pub fn if_else() -> Self {
        Op::Fn(
            "if_else",
            3.into(),
            Arc::new(|args| if args[0] { args[1] } else { args[2] }),
        )
    }

    pub fn and_then() -> Self {
        Op::Fn(
            "and_then",
            3.into(),
            Arc::new(|args| args[0] && args[1] && args[2]),
        )
    }

    pub fn or_else() -> Self {
        Op::Fn(
            "or_else",
            2.into(),
            Arc::new(|args| args[0] || args[1] || args[2]),
        )
    }

    pub fn nand() -> Self {
        Op::Fn("nand", 2.into(), Arc::new(|args| !(args[0] && args[1])))
    }

    pub fn nor() -> Self {
        Op::Fn("nor", 2.into(), Arc::new(|args| !(args[0] || args[1])))
    }

    pub fn xnor() -> Self {
        Op::Fn("xnor", 2.into(), Arc::new(|args| !(args[0] ^ args[1])))
    }

    pub fn implies() -> Self {
        Op::Fn("implies", 2.into(), Arc::new(|args| !args[0] || args[1]))
    }

    pub fn iff() -> Self {
        Op::Fn("iff", 2.into(), Arc::new(|args| args[0] == args[1]))
    }
}
