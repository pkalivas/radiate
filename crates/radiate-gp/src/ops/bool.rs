use super::Op;
use crate::ops::op_names;

/// One op per distinct boolean function, for building a [NodeSet](crate::NodeSet) that doesn't
/// favor any function by listing it twice: `and`, `or`, `not`, `xor`, `xnor`, `nand`, `nor`,
/// `implies` and `if_else`.
///
/// Left out on purpose: `gt`, `ge` and `lt`, which repeat a function with the arguments swapped
/// (`gt(a, b) = lt(b, a)`, `ge(a, b) = implies(b, a)`), and the three-input `and_then` /
/// `or_else`, which are nested `and` / `or`.
pub fn bool_ops() -> Vec<Op<bool>> {
    vec![
        Op::and(),
        Op::or(),
        Op::not(),
        Op::xor(),
        Op::xnor(),
        Op::nand(),
        Op::nor(),
        Op::implies(),
        Op::if_else(),
    ]
}

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

    pub fn gt() -> Self {
        Op::Fn(op_names::GT, 2.into(), |args: &[bool]| args[0] & !args[1])
    }

    pub fn ge() -> Self {
        Op::Fn(op_names::GE, 2.into(), |args: &[bool]| args[0] >= args[1])
    }

    pub fn lt() -> Self {
        Op::Fn(op_names::LT, 2.into(), |args: &[bool]| !args[0] & args[1])
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
        Op::Fn(op_names::OR_ELSE, 3.into(), |args: &[bool]| {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arity, Eval};

    /// Every op's output on every input combination for its arity.
    fn truth_table(op: &Op<bool>) -> Vec<bool> {
        let arity = match op.arity() {
            Arity::Exact(n) => n,
            other => panic!("{} has arity {other:?}", op.name()),
        };
        (0..1usize << arity)
            .map(|bits| {
                let inputs = (0..arity).map(|i| bits >> i & 1 == 1).collect::<Vec<_>>();
                op.eval(&inputs)
            })
            .collect()
    }

    #[test]
    fn bool_ops_are_all_distinct_functions() {
        let ops = bool_ops();
        for (i, a) in ops.iter().enumerate() {
            for b in &ops[i + 1..] {
                assert!(
                    a.arity() != b.arity() || truth_table(a) != truth_table(b),
                    "{} and {} compute the same function",
                    a.name(),
                    b.name()
                );
            }
        }
    }

    #[test]
    fn logic_ops_match_their_definitions() {
        let cases = [(false, false), (false, true), (true, false), (true, true)];
        for (a, b) in cases {
            assert_eq!(Op::<bool>::xor().eval(&[a, b]), a != b);
            assert_eq!(Op::<bool>::xnor().eval(&[a, b]), a == b);
            assert_eq!(Op::<bool>::implies().eval(&[a, b]), !a || b);
            assert_eq!(Op::<bool>::gt().eval(&[a, b]), a && !b);
            assert_eq!(Op::<bool>::lt().eval(&[a, b]), !a && b);
            assert_eq!(Op::<bool>::ge().eval(&[a, b]), a || !b);
        }
    }

    #[test]
    fn or_else_takes_three_inputs() {
        let op = Op::<bool>::or_else();
        assert_eq!(op.arity(), Arity::Exact(3));

        for a in [false, true] {
            for b in [false, true] {
                for c in [false, true] {
                    assert_eq!(op.eval(&[a, b, c]), a || b || c, "or_else({a}, {b}, {c})");
                }
            }
        }
    }
}
