pub use radiate_expr::*;
use radiate_utils::SmallStr;

const DEFAULT_VALUE: f32 = 1.0;

#[derive(Clone)]
pub struct RateSet {
    pub control: NamedExpr,
    pub internal: Vec<NamedExpr>,
}

impl RateSet {
    pub fn new(control: impl Into<NamedExpr>) -> Self {
        Self {
            control: control.into(),
            internal: Vec::new(),
        }
    }

    pub fn alias(mut self, name: impl Into<SmallStr>) -> Self {
        let name = name.into();
        self.control = self.control.expr.alias(name);
        self
    }

    pub fn add(&mut self, expr: impl Into<NamedExpr>) {
        self.internal.push(expr.into());
    }
}

impl Default for RateSet {
    fn default() -> Self {
        Self {
            control: NamedExpr::new("control", Expr::lit(DEFAULT_VALUE)),
            internal: Vec::new(),
        }
    }
}

// #[derive(Clone)]
// pub enum Rate {
//     Fixed(f32),
//     NamedExpr(NamedExpr),
// }

// impl Rate {
//     pub fn get(&mut self, metrics: &MetricSet) -> RadiateResult<f32> {
//         match self {
//             Rate::NamedExpr(named_expr) => match metrics.get(named_expr.name()) {
//                 Some(metric) => Ok(metric.last_value()),
//                 None => match named_expr.eval(metrics)?.extract::<f32>() {
//                     Some(value) => {
//                         if (0.0..=1.0).contains(&value) {
//                             Ok(value)
//                         } else {
//                             Err(radiate_err!(
//                                 Expr: "Expected f32 value between 0.0 and 1.0 from rate expression evaluation, but got {}",
//                                 value
//                             ))
//                         }
//                     }
//                     None => Err(radiate_err!(
//                         Expr: "Expected f32 value from rate expression evaluation, but got {:?}",
//                         named_expr.eval(metrics)?
//                     )),
//                 },
//             },
//             Rate::Fixed(value) => Ok(*value),
//         }
//     }

//     pub fn fixed_value(&self) -> f32 {
//         match self {
//             Rate::Fixed(value) => *value,
//             _ => DEFAULT_VALUE,
//         }
//     }
// }

// impl Valid for Rate {
//     fn is_valid(&self) -> bool {
//         match self {
//             Rate::Fixed(v) => (0.0..=1.0).contains(v),
//             _ => true,
//         }
//     }
// }

// impl From<f32> for Rate {
//     fn from(value: f32) -> Self {
//         Rate::Fixed(value)
//     }
// }

// impl From<NamedExpr> for Rate {
//     fn from(named_expr: NamedExpr) -> Self {
//         Rate::NamedExpr(named_expr)
//     }
// }

// // pub struct RateBuilder {
// //     pub name: SmallStr,
// //     pub exprs: ExprSet,
// // }

// // #[derive(Clone, Debug, PartialEq)]
// // pub enum CycleShape {
// //     Triangle,
// //     Sine,
// // }

// // /// Rate enum representing different types of rate schedules where each variant defines a
// // /// method to compute the rate value at a given step.
// // /// These are designed to produce values within the range [0.0, 1.0] - ie: a rate.
// // #[derive(Clone)]
// // pub enum Rate {
// //     Fixed(f32),
// //     Expr(Expr),
// //     NamedExpr(NamedExpr),
// //     Combined(Box<Rate>, Vec<Rate>),
// // }

// // impl Rate {
// //     pub fn sub_rate(mut self, sub_rate: impl Into<Rate>) -> Self {
// //         let sub_rate = sub_rate.into();
// //         match self {
// //             Rate::Combined(_, ref mut others) => {
// //                 others.push(sub_rate);
// //                 self
// //             }
// //             _ => Rate::Combined(Box::new(self), vec![sub_rate]),
// //         }
// //     }
// // }

// // impl Rate {
// //     pub fn get(&mut self, generation: usize, metrics: &MetricSet) -> f32 {
// //         match self {
// //             Rate::Expr(expr) => expr
// //                 .eval(metrics)
// //                 .ok()
// //                 .and_then(|v| v.extract())
// //                 .unwrap_or(0.0),
// //             _ => self.get_by_index(generation),
// //         }
// //     }

// //     pub fn get_by_index(&self, step: usize) -> f32 {
// //         match self {
// //             Rate::Fixed(v) => *v,
// //             _ => 1.0,
// //         }
// //     }
// // }

// // impl Valid for Rate {
// //     fn is_valid(&self) -> bool {
// //         match self {
// //             Rate::Fixed(v) => (0.0..=1.0).contains(v),

// //             _ => true,
// //         }
// //     }
// // }

// // impl Default for Rate {
// //     fn default() -> Self {
// //         Rate::Fixed(1.0)
// //     }
// // }

// // impl From<f32> for Rate {
// //     fn from(value: f32) -> Self {
// //         Rate::Fixed(value)
// //     }
// // }

// // impl From<Expr> for Rate {
// //     fn from(expr: Expr) -> Self {
// //         Rate::Expr(expr.compile())
// //     }
// // }

// // impl From<NamedExpr> for Rate {
// //     fn from(named_expr: NamedExpr) -> Self {
// //         Rate::NamedExpr(named_expr)
// //     }
// // }

// // impl Debug for Rate {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         match self {
// //             Rate::Fixed(v) => write!(f, "Rate::Fixed({})", v),
// //             Rate::Expr(_) => write!(f, "Rate::Expr(<function>)"),
// //             Rate::Combined(base, others) => {
// //                 write!(f, "Rate::Combined(base: {:?}, others: {:?})", base, others)
// //             }
// //             Rate::NamedExpr(named_expr) => {
// //                 write!(
// //                     f,
// //                     "Rate::NamedExpr(name: {}, expr: {:?})",
// //                     named_expr.name, named_expr.expr
// //                 )
// //             }
// //         }
// //     }
// // }

// // impl PartialEq for Rate {
// //     fn eq(&self, other: &Self) -> bool {
// //         match (self, other) {
// //             (Rate::Fixed(a), Rate::Fixed(b)) => a == b,
// //             // For Expr variants, we consider them equal if they are the same variant,
// //             // since we cannot compare the inner function for equality.
// //             (Rate::Expr(_), Rate::Expr(_)) => true,
// //             (Rate::NamedExpr(a), Rate::NamedExpr(b)) => a.name == b.name,
// //             (Rate::Combined(a_base, a_others), Rate::Combined(b_base, b_others)) => {
// //                 a_base == b_base && a_others == b_others
// //             }
// //             _ => false,
// //         }
// //     }
// // }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;

// //     #[test]
// //     fn test_rate_values() {
// //         let fixed = Rate::Fixed(0.5);
// //         assert_eq!(fixed.get_by_index(0), 0.5);
// //         assert_eq!(fixed.get_by_index(10), 0.5);
// //     }

// //     #[test]
// //     fn test_default_rate() {
// //         let default_rate = Rate::default();
// //         assert_eq!(default_rate.get_by_index(0), 1.0);
// //         assert_eq!(default_rate.get_by_index(100), 1.0);
// //     }

// //     #[test]
// //     fn test_rate_validity() {
// //         let valid_fixed = Rate::Fixed(0.5);
// //         let invalid_fixed = Rate::Fixed(1.5);
// //         assert!(valid_fixed.is_valid());
// //         assert!(!invalid_fixed.is_valid());
// //     }
// // }

// // // Rate::Linear(start, end, steps) => {
// // //     if *steps == 0 {
// // //         return *end;
// // //     }

// // //     let t = (f_step / *steps as f32).min(1.0);
// // //     start + (end - start) * t
// // // }
// // // Rate::Exponential(start, end, half_life) => {
// // //     if *half_life == 0 {
// // //         return *end;
// // //     }

// // //     let decay = 0.5_f32.powf(f_step / *half_life as f32);
// // //     end + (start - end) * decay
// // // }
// // // Rate::Cyclical(min, max, period, shape) => {
// // //     let phase = (f_step % *period as f32) / *period as f32;
// // //     let tri = if phase < 0.5 {
// // //         phase * 2.0
// // //     } else {
// // //         (1.0 - phase) * 2.0
// // //     };

// // //     let s = match shape {
// // //         CycleShape::Triangle => tri,
// // //         CycleShape::Sine => (std::f32::consts::TAU * phase).sin().abs(),
// // //     };

// // //     min + (max - min) * s
// // // }
// // // Rate::Stepwise(steps) => {
// // //     if steps.is_empty() {
// // //         return 0.0;
// // //     }

// // //     let mut last_value = steps[0].1;
// // //     for (s, v) in steps {
// // //         if step < *s {
// // //             break;
// // //         }

// // //         last_value = *v;
// // //     }

// // //     last_value
// // // }
