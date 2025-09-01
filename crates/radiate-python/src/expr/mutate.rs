use crate::{CrossoverExpr, ExprNode, ExprValue, MutateExpr, value::apply_numeric_slot_mut};
use radiate::{ArithmeticGene, Gene, chromosomes::gene::NumericSlotMut, random_provider};

impl MutateExpr {
    pub fn apply_numeric_mutate<G: ArithmeticGene>(&self, input: &mut G) -> usize {
        let mut changed = false;
        match self {
            MutateExpr::Uniform(amount) => {
                input.numeric_slot_mut().map(|slot| {
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
                input.numeric_slot_mut().map(|slot| {
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
                input.numeric_slot_mut().map(|slot| {
                    println!("Jittering slot:");
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

fn set_both_mean(a: &mut NumericSlotMut<'_>, b: &mut NumericSlotMut<'_>) -> bool {
    match (a, b) {
        (NumericSlotMut::F32(aa), NumericSlotMut::F32(bb)) => {
            let m = (**aa + **bb) * 0.5;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::F64(aa), NumericSlotMut::F64(bb)) => {
            let m = (**aa + **bb) * 0.5;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::I8(aa), NumericSlotMut::I8(bb)) => {
            let m = ((**aa as i32 + **bb as i32) / 2) as i8;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::I16(aa), NumericSlotMut::I16(bb)) => {
            let m = ((**aa as i32 + **bb as i32) / 2) as i16;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::I32(aa), NumericSlotMut::I32(bb)) => {
            let m = ((**aa as i64 + **bb as i64) / 2) as i32;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::I64(aa), NumericSlotMut::I64(bb)) => {
            let m = ((**aa as i128 + **bb as i128) / 2) as i64;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::U8(aa), NumericSlotMut::U8(bb)) => {
            let m = ((**aa as u32 + **bb as u32) / 2) as u8;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::U16(aa), NumericSlotMut::U16(bb)) => {
            let m = ((**aa as u32 + **bb as u32) / 2) as u16;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::U32(aa), NumericSlotMut::U32(bb)) => {
            let m = ((**aa as u64 + **bb as u64) / 2) as u32;
            **aa = m;
            **bb = m;
            true
        }
        (NumericSlotMut::U64(aa), NumericSlotMut::U64(bb)) => {
            let m = ((**aa >> 1) + (**bb >> 1) + ((**aa & **bb) & 1));
            **aa = m;
            **bb = m;
            true
        }
        _ => false,
    }
}

// ===== CrossoverExpr: elementwise swap/slice-swap/mean for Pair & SequencePair =====
impl CrossoverExpr {
    pub fn apply_crossover<'a, T: ExprNode>(&self, input: ExprValue<'a, T>) -> usize {
        match input {
            ExprValue::Pair(a, b) => self.apply_pair_nodes(a, b),
            ExprValue::SequencePair(a, b) => self.apply_pair_slices(a, b),
            _ => 0,
        }
    }

    fn apply_pair_slices<T: ExprNode>(&self, a: &mut [T], b: &mut [T]) -> usize {
        let n = a.len().min(b.len());
        match self {
            CrossoverExpr::OnePoint => {
                let len1 = a.len();
                let len2 = b.len();
                let n = len1.min(len2);
                if n == 0 {
                    return 0;
                }

                let cut = if n == 1 {
                    0
                } else {
                    random_provider::range(0..n)
                };
                let tail_len = n - cut;

                let (_, tail1) = a.split_at_mut(cut);
                let (_, tail2) = b.split_at_mut(cut);

                let tail1 = &mut tail1[..tail_len];
                let tail2 = &mut tail2[..tail_len];

                tail1.swap_with_slice(tail2);
                return tail1.len() + tail2.len();
            }
            CrossoverExpr::TwoPoint => {
                if n < 2 {
                    return 0;
                }
                let mut i = random_provider::range(0..n);
                let mut j = random_provider::range(0..n);
                if i > j {
                    std::mem::swap(&mut i, &mut j);
                }
                for k in i..j {
                    std::mem::swap(&mut a[k], &mut b[k]);
                }
                j.saturating_sub(i)
            }
            CrossoverExpr::Swap => {
                if n == 0 {
                    return 0;
                }
                let i = random_provider::range(0..n);
                std::mem::swap(&mut a[i], &mut b[i]);
                1
            }
            CrossoverExpr::Mean => {
                let mut changed = 0;
                for i in 0..n {
                    if let (Some(mut sa), Some(mut sb)) = (a[i].numeric_mut(), b[i].numeric_mut()) {
                        if set_both_mean(&mut sa, &mut sb) {
                            changed += 1;
                        }
                    }
                }
                changed
            }
        }
    }

    fn apply_pair_nodes<T: ExprNode>(&self, a: &mut T, b: &mut T) -> usize {
        match self {
            CrossoverExpr::Swap => {
                std::mem::swap(a, b);
                1
            }
            CrossoverExpr::Mean => {
                if let (Some(mut sa), Some(mut sb)) = (a.numeric_mut(), b.numeric_mut()) {
                    return if set_both_mean(&mut sa, &mut sb) {
                        1
                    } else {
                        0
                    };
                }
                0
            }
            _ => 0, // OnePoint/TwoPoint don't apply to a single pair of nodes
        }
    }
}

impl MutateExpr {
    pub fn apply_mutator<N: ExprNode>(&self, input: &mut N) -> usize {
        let mut changed = false;

        match self {
            MutateExpr::Uniform(amount) => {
                input.numeric_mut().map(|slot| {
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
                input.numeric_mut().map(|slot| {
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
                input.numeric_mut().map(|slot| {
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
