use crate::{AnyValue, DataType, ExprNode, ExprNodePair, LeafView, PairLeafView};
use radiate::random_provider;
use std::ops::Range;

pub trait Eval<I: ?Sized, O> {
    fn eval(&self, input: &I) -> O;
}

pub trait EvalMut<I: ?Sized, O> {
    fn eval_mut(&mut self, input: &I) -> O;
}

impl<I: ?Sized, O> EvalMut<I, O> for dyn Eval<I, O> {
    fn eval_mut(&mut self, input: &I) -> O {
        self.eval(input)
    }
}

#[derive(Debug, Clone)]
pub enum MutateExpr {
    Uniform(Range<f32>),
    Gaussian(f32, f32),
    Jitter(f32),
}

#[derive(Debug, Clone)]
pub enum CrossoverExpr {
    OnePoint,
    TwoPoint,
    Swap,
    Mean,
}

#[derive(Debug, Clone)]
pub struct Alteration {
    pub name: String,
    pub expr: Expr,
    pub target: String,
    pub p: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprPath<'a> {
    Root,
    Field(&'a str),
    Index(usize),
}

#[derive(Debug, Clone)]
pub enum Expr {
    // structure/navigation
    This,                       // do nothing
    AtField(String, Box<Expr>), // run inner where path == field
    AtIndex(usize, Box<Expr>),  // run inner at a specific index
    All(Box<Expr>),             // map inner across all children (vectors/structs)

    // combinators
    Seq(Vec<Expr>),             // run in order (pipe)
    Prob(f32, Box<Expr>),       // run inner with probability p
    DType(DataType, Box<Expr>), // run inner only if leaf dtype matches

    // leaf ops
    Mut(MutateExpr),
    Cross(CrossoverExpr), // used by pair eval
}

// impl<N: ExprNode

impl Expr {
    pub fn eval_on<'a, N: ExprNode<AnyValue<'a>>>(&self, node: &mut N) -> usize {
        let mut changed = 0;

        match self {
            Expr::This => {}
            Expr::Seq(list) => {
                for e in list {
                    changed += e.eval_on(node);
                }
            }
            Expr::Prob(p, inner) => {
                if *p <= 0.0 {
                    return 0;
                }

                if random_provider::random::<f32>() < *p {
                    changed += inner.eval_on(node);
                }
            }
            // Expr::DType(dt, inner) => {
            //     node.visit(&mut |_, value| {
            //         if value.dtype() == *dt {
            //             changed += inner.eval_on_leaf(&mut leaf);
            //         }
            //     });
            // }
            // Expr::AtField(name, inner) => {
            //     let target = name.as_str();
            //     node.visit(&mut |path, value| {
            //         let mut leaf = LeafView::from(value);
            //         if matches!(path, ExprPath::Field(n) if n == target) {
            //             changed += inner.eval_on_leaf(&mut leaf);
            //         }
            //     });
            // }
            // Expr::AtIndex(i, inner) => {
            //     node.visit(&mut |path, LeafView::from(value)| {
            //         if matches!(path, ExprPath::Index(j) if j == *i) {
            //             changed += inner.eval_on_leaf(value);
            //         }
            //     });
            // }
            // Expr::All(inner) => {
            //     node.visit(&mut |_, LeafView::from(value)| {
            //         changed += inner.eval_on_leaf(value);
            //     });
            // }
            // Expr::Mut(m) => {
            //     node.visit(&mut |_, LeafView::from(value)| {
            //         changed += m.apply(value);
            //     });
            // }
            Expr::Cross(_) => {
                // no-op in mutate mode
            }
            _ => {
                // unhandled cases
            }
        }

        changed
    }

    pub(crate) fn eval_on_leaf(&self, v: &mut AnyValue<'_>) -> usize {
        match self {
            Expr::Mut(m) => m.apply(v),
            Expr::Seq(list) => list.iter().fold(0, |acc, e| acc + e.eval_on_leaf(v)),
            Expr::Prob(p, inner) => {
                if *p <= 0.0 || *p > 1.0 {
                    return 0;
                } else if random_provider::random::<f32>() < *p {
                    return inner.eval_on_leaf(v);
                } else {
                    return 0;
                }
            }
            Expr::DType(dt, inner) => {
                if v.dtype() == *dt {
                    inner.eval_on_leaf(v)
                } else {
                    0
                }
            }
            // navigation nodes are handled by the visitor layer above,
            // so at a leaf they do nothing:
            _ => 0,
        }
    }

    pub fn eval_pair_on_leaves<'a>(&self, a: &mut AnyValue<'a>, b: &mut AnyValue<'a>) -> usize {
        match self {
            Expr::Cross(CrossoverExpr::Swap) => swap_leaf(a, b) as usize,
            Expr::Cross(CrossoverExpr::Mean) => mean_leaf(a, b) as usize,
            // You can allow mutation-on-both in crossover if desired:
            // Expr::Mut(m) => m.apply(a) + m.apply(b),
            Expr::Seq(list) => list
                .iter()
                .fold(0, |acc, e| acc + e.eval_pair_on_leaves(a, b)),
            Expr::Prob(p, inner) => {
                if *p <= 0.0 || *p > 1.0 {
                    return 0;
                } else if random_provider::random::<f32>() < *p {
                    return inner.eval_pair_on_leaves(a, b);
                } else {
                    return 0;
                }
            }
            // path/dtype filters: apply to both sides when they match
            Expr::DType(dt, inner) => {
                if a.dtype() == *dt && b.dtype() == *dt {
                    inner.eval_pair_on_leaves(a, b)
                } else {
                    0
                }
            }
            // AtField/AtIndex would be handled *above*, where you know which subtree you're pairing.
            _ => 0,
        }
    }

    pub(crate) fn eval_on_pair<'a>(&self, one: &mut AnyValue<'a>, two: &mut AnyValue<'a>) -> usize {
        match self {
            Expr::Cross(c) => match c {
                CrossoverExpr::OnePoint | CrossoverExpr::TwoPoint => 0,
                CrossoverExpr::Swap => {
                    let mut count = 0;
                    (one, two).visit_pair(&mut |_, PairLeafView { left, right }| {
                        count += self.eval_pair_on_leaves(left, right);
                    });
                    count
                }
                CrossoverExpr::Mean => {
                    let mut count = 0;
                    (one, two).visit_pair(&mut |_, PairLeafView { left, right }| {
                        count += self.eval_pair_on_leaves(left, right);
                    });
                    count
                }
            },
            Expr::Seq(list) => list.iter().fold(0, |acc, e| acc + e.eval_on_pair(one, two)),
            Expr::Prob(p, inner) => {
                if *p <= 0.0 || *p > 1.0 {
                    return 0;
                } else if random_provider::random::<f32>() < *p {
                    return inner.eval_on_pair(one, two);
                } else {
                    return 0;
                }
            }
            Expr::DType(dt, inner) => {
                if one.dtype() == *dt && two.dtype() == *dt {
                    inner.eval_on_pair(one, two)
                } else {
                    0
                }
            }
            // navigation nodes are handled by the visitor layer above,
            // so at a leaf they do nothing:
            _ => 0,
        }
    }
}

impl Alteration {
    pub fn new(name: String, expr: Expr, target: String, p: f32) -> Self {
        Alteration {
            name,
            expr,
            target,
            p,
        }
    }
}

fn swap_leaf<'a>(a: &mut AnyValue<'a>, b: &mut AnyValue<'a>) -> bool {
    std::mem::swap(a, b);
    true
}

fn mean_leaf<'a>(a: &mut AnyValue<'a>, b: &mut AnyValue<'a>) -> bool {
    use AnyValue::*;
    match (a, b) {
        (Float32(aa), Float32(bb)) => {
            let m = (*aa + *bb) * 0.5;
            *aa = m;
            true
        }
        (Float64(aa), Float64(bb)) => {
            let m = (*aa + *bb) * 0.5;
            *aa = m;
            true
        }
        (Int8(aa), Int8(bb)) => {
            let m = ((*aa as i32 + *bb as i32) / 2) as i8;
            *aa = m;
            true
        }
        (Int16(aa), Int16(bb)) => {
            let m = ((*aa as i32 + *bb as i32) / 2) as i16;
            *aa = m;
            true
        }
        (Int32(aa), Int32(bb)) => {
            let m = ((*aa as i64 + *bb as i64) / 2) as i32;
            *aa = m;
            true
        }
        (Int64(aa), Int64(bb)) => {
            let m = ((*aa as i128 + *bb as i128) / 2) as i64;
            *aa = m;
            true
        }
        (UInt8(aa), UInt8(bb)) => {
            let m = ((*aa as u32 + *bb as u32) / 2) as u8;
            *aa = m;
            true
        }
        (UInt16(aa), UInt16(bb)) => {
            let m = ((*aa as u32 + *bb as u32) / 2) as u16;
            *aa = m;
            true
        }
        (UInt32(aa), UInt32(bb)) => {
            let m = ((*aa as u64 + *bb as u64) / 2) as u32;
            *aa = m;
            true
        }
        (UInt64(aa), UInt64(bb)) => {
            // avoid overflow: average as (a/2 + b/2 + carry)
            let m = (*aa >> 1) + (*bb >> 1) + ((*aa & *bb) & 1);
            *aa = m;
            true
        }
        _ => false,
    }
}

impl MutateExpr {
    pub fn apply(&self, value: &mut AnyValue<'_>) -> usize {
        let mut changed = false;
        match self {
            MutateExpr::Uniform(amount) => {
                value.with_numeric_mut(|slot| {
                    crate::value::apply_numeric_slot_mut(
                        slot,
                        |x_f32| {
                            let delta = random_provider::range(amount.clone());
                            x_f32 + delta
                        },
                        |x_f64| {
                            let delta = random_provider::range(amount.clone()) as f64;
                            x_f64 + delta
                        },
                        |i, unsigned| {
                            // Sample f32 delta from range, round to nearest int
                            let delta = random_provider::range(amount.clone()).round() as i128;
                            let y = i.saturating_add(delta);
                            if unsigned { y.max(0) } else { y }
                        },
                    );

                    changed = true;
                });
            }
            MutateExpr::Gaussian(mean, stddev) => {
                let mu = *mean as f64;
                let sd = (*stddev as f64).max(1e-12);
                value.with_numeric_mut(|slot| {
                    crate::value::apply_numeric_slot_mut(
                        slot,
                        |x_f32| {
                            let delta = random_provider::gaussian(mu, sd) as f32;
                            x_f32 + delta
                        },
                        |x_f64| {
                            let delta = random_provider::gaussian(mu, sd);
                            x_f64 + delta
                        },
                        |i, unsigned| {
                            // Integer: gaussian delta rounded to nearest int
                            let delta = random_provider::gaussian(mu, sd).round() as i128;
                            let y = i.saturating_add(delta);
                            if unsigned { y.max(0) } else { y }
                        },
                    );
                    changed = true;
                });
            }
            MutateExpr::Jitter(frac) => {
                let frac = *frac as f64;
                value.with_numeric_mut(|slot| {
                    crate::value::apply_numeric_slot_mut(
                        slot,
                        |x_f32| {
                            let change = random_provider::range(-1.0..1.0) * frac;
                            x_f32 + change as f32
                        },
                        |x_f64| {
                            let change = random_provider::gaussian(0.0, frac * x_f64.abs());
                            x_f64 + change
                        },
                        |i, unsigned| {
                            // Integer: gaussian change rounded to nearest int
                            let change = random_provider::gaussian(0.0, frac * (i.abs() as f64))
                                .round() as i128;
                            let y = i.saturating_add(change);
                            if unsigned { y.max(0) } else { y }
                        },
                    );

                    changed = true;
                });
            }
        }

        return if changed { 1 } else { 0 };
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let alteration = Alteration::new(
            "test".into(),
            Expr::Mut(MutateExpr::Uniform(-100.0..100.0)),
            "target".into(),
            0.99,
        );

        let gaussian_alteration = Alteration::new(
            "test".into(),
            Expr::Mut(MutateExpr::Jitter(0.1)),
            "target".into(),
            1.0,
        );

        let mut value = AnyValue::Float32(1.0);

        alteration.expr.eval_on(&mut value);
        println!("Value after alteration: {:?}", value);

        let mut nested_value = AnyValue::Struct(vec![
            (AnyValue::StrOwned("a".into()), "a".into()),
            (AnyValue::Float32(2.0), "other".into()),
            (
                AnyValue::Vector(Box::new(vec![
                    AnyValue::Float32(3.0),
                    AnyValue::Struct(vec![
                        (AnyValue::StrOwned("t".into()), "a".into()),
                        (AnyValue::Float32(4.0), "target".into()),
                    ]),
                ])),
                "list".into(),
            ),
        ]);

        println!("Nested value after alteration: {:?}", nested_value);
        gaussian_alteration.expr.eval_on(&mut nested_value);
        println!("Nested value after alteration: {:?}", nested_value);
    }
}
