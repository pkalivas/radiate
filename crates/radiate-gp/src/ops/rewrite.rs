use std::sync::Arc;

use crate::collections::trees::TreeNode;
use crate::ops::{Op, op_names};
use crate::{Node, TreeRewriterRule};

pub struct OpTreeRewriteRule<T> {
    pub apply: Arc<dyn for<'a> Fn(&'a mut TreeNode<Op<T>>) -> bool>,
}

impl<T> OpTreeRewriteRule<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: for<'a> Fn(&'a mut TreeNode<Op<T>>) -> bool + 'static,
    {
        OpTreeRewriteRule { apply: Arc::new(f) }
    }
}

impl TreeRewriterRule<Op<f32>> for OpTreeRewriteRule<f32> {
    fn apply<'a>(&self, node: &'a mut TreeNode<Op<f32>>) -> bool {
        (self.apply)(node)
    }
}

pub fn all_rewrite_rules() -> Vec<OpTreeRewriteRule<f32>> {
    let mut rules = Vec::new();

    rules.extend(neutral_add_sub_mul_div());
    rules.extend(fold_add_sub_mul_div());
    rules.extend(neg_rules());
    rules.extend(sum_prod_rules());

    rules
}

fn is_zero(n: &TreeNode<Op<f32>>) -> bool {
    match n.value() {
        Op::Const(_, v) => v.abs() <= std::f32::EPSILON,
        _ => false,
    }
}

fn is_one(n: &TreeNode<Op<f32>>) -> bool {
    match n.value() {
        Op::Const(_, v) => (*v - crate::ops::math::ONE).abs() <= std::f32::EPSILON,
        _ => false,
    }
}

// Replace current node with one of its children by moving it, no clone.
// idx must be valid and children must exist. This discards the other child subtree (intended).
fn replace_with_child_idx(node: &mut TreeNode<Op<f32>>, idx: usize) -> bool {
    if let Some(children) = node.children_mut() {
        if idx < children.len() {
            let mut subtree = children.swap_remove(idx);
            std::mem::swap(node, &mut subtree);
            return true;
        }
    }
    false
}

fn replace_with_const(node: &mut TreeNode<Op<f32>>, name: &'static str, v: f32) -> bool {
    let mut new_leaf = TreeNode::new(Op::Const(name, v));
    std::mem::swap(node, &mut new_leaf);
    true
}

// Neutral/identity rules (in-place; no subtree clones)
pub fn neutral_add_sub_mul_div() -> Vec<OpTreeRewriteRule<f32>> {
    vec![
        // add(x,0) or add(0,x) -> x
        OpTreeRewriteRule::new(|n| {
            match n.value() {
                Op::Fn(name, _, _) if *name == op_names::ADD => {
                    if let Some(children) = n.children_mut() {
                        if children.len() == 2 {
                            if is_zero(&children[0]) {
                                return replace_with_child_idx(n, 1);
                            }
                            if is_zero(&children[1]) {
                                return replace_with_child_idx(n, 0);
                            }
                        }
                    }
                }
                _ => {}
            }

            false
        }),
        // sub(x,0) -> x
        OpTreeRewriteRule::new(|n| {
            match n.value() {
                Op::Fn(name, _, _) if *name == op_names::SUB => {
                    if let Some(children) = n.children_mut() {
                        if children.len() == 2 && is_zero(&children[1]) {
                            return replace_with_child_idx(n, 0);
                        }
                    }
                }
                _ => {}
            }

            false
        }),
        // sub(x,x) -> 0
        OpTreeRewriteRule::new(|n| {
            match n.value() {
                Op::Fn(name, _, _) if *name == op_names::SUB => {
                    if let Some(children) = n.children() {
                        if children.len() == 2 && children[0] == children[1] {
                            return replace_with_const(n, "0", 0.0);
                        }
                    }
                }
                _ => {}
            }

            false
        }),
        // mul(x,1) or mul(1,x) -> x
        OpTreeRewriteRule::new(|n| {
            match n.value() {
                Op::Fn(name, _, _) if *name == op_names::MUL => {
                    if let Some(children) = n.children_mut() {
                        if children.len() == 2 {
                            if is_one(&children[0]) {
                                return replace_with_child_idx(n, 1);
                            }
                            if is_one(&children[1]) {
                                return replace_with_child_idx(n, 0);
                            }
                        }
                    }
                }
                _ => {}
            }

            false
        }),
        // mul(x,0) or mul(0,x) -> 0
        OpTreeRewriteRule::new(|n| {
            match n.value() {
                Op::Fn(name, _, _) if *name == op_names::MUL => {
                    if let Some(children) = n.children() {
                        if children.len() == 2 && (is_zero(&children[0]) || is_zero(&children[1])) {
                            return replace_with_const(n, "0", 0.0);
                        }
                    }
                }
                _ => {}
            }

            false
        }),
        // div(x,1) -> x
        OpTreeRewriteRule::new(|n| {
            match n.value() {
                Op::Fn(name, _, _) if *name == op_names::DIV => {
                    if let Some(children) = n.children_mut() {
                        if children.len() == 2 && is_one(&children[1]) {
                            return replace_with_child_idx(n, 0);
                        }
                    }
                }
                _ => {}
            }

            false
        }),
    ]
}

pub fn fold_add_sub_mul_div() -> Vec<OpTreeRewriteRule<f32>> {
    let fold = |name: &'static str, f: fn(f32, f32) -> f32| {
        OpTreeRewriteRule::new(move |n| {
            if let Op::Fn(op_name, _, _) = n.value() {
                if *op_name == name {
                    if let Some(children) = n.children() {
                        if children.len() == 2 {
                            match (children[0].value(), children[1].value()) {
                                (Op::Const(_, a), Op::Const(_, b)) => {
                                    return replace_with_const(n, "c", f(*a, *b));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            false
        })
    };

    vec![
        fold(op_names::ADD, |a, b| a + b),
        fold(op_names::SUB, |a, b| a - b),
        fold(op_names::MUL, |a, b| a * b),
        fold(op_names::DIV, |a, b| a / b),
    ]
}

pub fn neg_rules() -> Vec<OpTreeRewriteRule<f32>> {
    vec![
        // neg(neg(x)) -> x
        OpTreeRewriteRule::new(|n| {
            if let Op::Fn(name, _, _) = n.value() {
                if *name == op_names::NEG {
                    if let Some(children) = n.children() {
                        if children.len() >= 1 {
                            if let Op::Fn(n2, _, _) = children[0].value() {
                                if *n2 == op_names::NEG {
                                    // move the grandchild into place
                                    if let Some(grand) = children[0].children() {
                                        if let Some(_) = grand.get(0) {
                                            // swap with a moved copy of grandchild (avoid clone via take_children):
                                            // use take from parent:
                                            if let Some(mut cs) = n.take_children() {
                                                if cs.len() == 1 {
                                                    if let Some(mut gs) = cs[0].take_children() {
                                                        if !gs.is_empty() {
                                                            let mut only = gs.swap_remove(0);
                                                            std::mem::swap(n, &mut only);
                                                            return true;
                                                        }
                                                    }
                                                }
                                                // restore if failed
                                                n.add_child(cs.swap_remove(0));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            false
        }),
        // neg(Const) -> Const(-v)
        OpTreeRewriteRule::new(|n| {
            if let Op::Fn(name, _, _) = n.value() {
                if *name == op_names::NEG {
                    if let Some(children) = n.children() {
                        if children.len() == 1 {
                            if let Op::Const(_, v) = children[0].value() {
                                return replace_with_const(n, "c", -*v);
                            }
                        }
                    }
                }
            }

            false
        }),
    ]
}

pub fn sum_prod_rules() -> Vec<OpTreeRewriteRule<f32>> {
    vec![
        // sum(...,0,...) -> drop zeros; unwrap; empty->0
        OpTreeRewriteRule::new(|n| {
            if let Op::Fn(name, _, _) = n.value() {
                if *name == op_names::SUM {
                    if let Some(mut cs) = n.take_children() {
                        let mut kept = Vec::with_capacity(cs.len());
                        let mut dropped = false;
                        while let Some(ch) = cs.pop() {
                            if is_zero(&ch) {
                                dropped = true;
                            } else {
                                kept.push(ch);
                            }
                        }
                        kept.reverse();
                        if kept.is_empty() {
                            return replace_with_const(n, "0", 0.0);
                        }
                        if kept.len() == 1 {
                            let mut only = kept.swap_remove(0);
                            std::mem::swap(n, &mut only);
                            return true;
                        }
                        if dropped {
                            // put back pruned children
                            for k in kept {
                                n.add_child(k);
                            }
                            return true;
                        } else {
                            // nothing changed; restore original children
                            for c in kept {
                                n.add_child(c);
                            }
                        }
                    }
                }
            }
            false
        }),
        // prod: zero short-circuit; drop ones; unwrap; empty->1
        OpTreeRewriteRule::new(|n| {
            if let Op::Fn(name, _, _) = n.value() {
                if *name == op_names::PROD {
                    if let Some(mut cs) = n.take_children() {
                        let mut kept = Vec::with_capacity(cs.len());
                        while let Some(ch) = cs.pop() {
                            if is_zero(&ch) {
                                return replace_with_const(n, "0", 0.0);
                            }
                            if is_one(&ch) {
                                continue;
                            }
                            kept.push(ch);
                        }
                        kept.reverse();
                        if kept.is_empty() {
                            return replace_with_const(n, "1", 1.0);
                        }
                        if kept.len() == 1 {
                            let mut only = kept.swap_remove(0);
                            std::mem::swap(n, &mut only);
                            return true;
                        }
                        if let Some(_) = n.children_mut() {
                            for k in kept {
                                n.add_child(k);
                            }
                            return true;
                        }
                    }
                }
            }
            false
        }),
    ]
}

// Post-order application (in-place). Returns number of rewrites.
pub fn apply_rules_once(root: &mut TreeNode<Op<f32>>, rules: &[OpTreeRewriteRule<f32>]) -> usize {
    let mut count = 0;

    if let Some(children) = root.children_mut() {
        for child in children.iter_mut() {
            count += apply_rules_once(child, rules);
        }
    }

    #[cfg(feature = "pgm")]
    {
        if let Op::PGM(_, _, programs, _) = root.value_mut() {
            let progs = Arc::make_mut(programs);
            for p in progs.iter_mut() {
                count += apply_rules_once(p, rules);
            }
        }
    }

    for rule in rules {
        if (rule.apply)(root) {
            count += 1;
            break;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_rules_once() {
        let mut root = TreeNode::new(Op::add())
            .attach(Op::named_constant("x", 1.0))
            .attach(Op::named_constant("0", 0.0));

        let rules = neutral_add_sub_mul_div();

        let count = apply_rules_once(&mut root, &rules);
        assert_eq!(count, 1);
        assert_eq!(
            match root.value() {
                Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            },
            1.0
        );
    }

    #[test]
    fn test_fold_add_sub_mul_div() {
        let mut root = TreeNode::new(Op::add())
            .attach(Op::named_constant("x", 1.0))
            .attach(Op::named_constant("y", 2.0));

        let rules = fold_add_sub_mul_div();

        let count = apply_rules_once(&mut root, &rules);
        assert_eq!(count, 1);
        assert_eq!(
            match root.value() {
                Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            },
            3.0
        );
    }

    #[test]
    fn test_neg_rules() {
        // Build neg(neg(x))
        let mut root = TreeNode::new(Op::neg())
            .attach(TreeNode::new(Op::neg()).attach(Op::named_constant("x", 3.0)));

        let rules = neg_rules();
        let count = apply_rules_once(&mut root, &rules);

        assert_eq!(count, 2);
        match root.value() {
            Op::Const(_, v) => assert_eq!(*v, 3.0),
            _ => panic!("Expected constant"),
        }
    }

    #[test]
    fn test_sum_prod_rules() {
        let mut root = TreeNode::new(Op::sum())
            .attach(Op::named_constant("x", 2.0))
            .attach(Op::named_constant("0", 0.0))
            .attach(Op::named_constant("y", 3.0));
        let rules = sum_prod_rules();
        let count = apply_rules_once(&mut root, &rules);

        assert_eq!(count, 1);
        assert_eq!(root.children().unwrap().len(), 2);
        assert_eq!(
            match root.children().unwrap()[0].value() {
                Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            },
            2.0
        );
        assert_eq!(
            match root.children().unwrap()[1].value() {
                Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            },
            3.0
        );
    }
}
