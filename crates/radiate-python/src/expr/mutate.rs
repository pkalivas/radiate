use crate::{ExprNode, MutateExpr, apply_numeric_slot_mut};
use radiate::random_provider;

impl MutateExpr {
    pub fn apply_numeric_mutate<G: ExprNode>(&self, input: &mut G) -> usize {
        let mut changed = false;
        match self {
            MutateExpr::Uniform(amount) => {
                input.numeric_mut().map(|slot| {
                    apply_numeric_slot_mut(
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
                    apply_numeric_slot_mut(
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
                    apply_numeric_slot_mut(
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

impl MutateExpr {
    pub fn apply_mutator<N: ExprNode>(&self, input: &mut N) -> usize {
        let mut changed = false;

        match self {
            MutateExpr::Uniform(amount) => {
                input.numeric_mut().map(|slot| {
                    apply_numeric_slot_mut(
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
                    apply_numeric_slot_mut(
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
                    apply_numeric_slot_mut(
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
