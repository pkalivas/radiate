use crate::{Alteration, AnyChromosome, AnyValue, CrossoverExpr, DataType, Expr, MutateExpr};
use pyo3::{pyclass, pymethods};
use radiate::{AlterResult, Chromosome, Crossover, Mutate, random_provider};
use std::ops::Range;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyAlteration {
    pub inner: Alteration,
}

#[pymethods]
impl PyAlteration {
    #[staticmethod]
    #[pyo3(signature = (range, target, p=1.0))]
    fn uniform(range: (f32, f32), target: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "uniform".into(),
                Expr::Mut(MutateExpr::Uniform(range.0..range.1)),
                target,
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (mean, stddev, target, p=1.0))]
    fn gaussian(mean: f32, stddev: f32, target: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "gaussian".into(),
                Expr::Mut(MutateExpr::Gaussian(mean, stddev)),
                target,
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (amount, target, p=1.0))]
    fn jitter(amount: f32, target: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "jitter".into(),
                Expr::Mut(MutateExpr::Jitter(amount)),
                target,
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn swap(target: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new("swap".into(), Expr::Cross(CrossoverExpr::Swap), target, p),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn mean(target: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new("mean".into(), Expr::Cross(CrossoverExpr::Mean), target, p),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn one_point(target: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "one_point".into(),
                Expr::Cross(CrossoverExpr::OnePoint),
                target,
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, p=1.0))]
    fn two_point(target: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "two_point".into(),
                Expr::Cross(CrossoverExpr::TwoPoint),
                target,
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (field_name, inner))]
    fn at_field(field_name: String, inner: &PyAlteration) -> Self {
        PyAlteration {
            inner: Alteration::new(
                format!("at_field({})", field_name),
                Expr::AtField(field_name, Box::new(inner.inner.expr.clone())),
                inner.inner.target.clone(),
                inner.inner.p,
            ),
        }
    }
}

pub struct ExprMutator {
    pub alters: Vec<Alteration>,
}

impl ExprMutator {
    pub fn new(alters: Vec<Alteration>) -> Self {
        ExprMutator { alters }
    }
}

impl Mutate<AnyChromosome<'_>> for ExprMutator {
    fn name(&self) -> String {
        "ExpressionMutator".into()
    }

    fn mutate_chromosome(&self, chromosome: &mut AnyChromosome<'_>, _: f32) -> AlterResult {
        let mut count = 0;

        for alteration in &self.alters {
            for gene in chromosome.iter_mut() {
                if random_provider::random::<f32>() < alteration.p {
                    // count += alteration.expr.apply(gene.allele_mut()) as usize;
                }
            }
        }

        count.into()
    }
}

pub struct ExprCrossover {
    pub alters: Vec<Alteration>,
}

impl ExprCrossover {
    pub fn new(alters: Vec<Alteration>) -> Self {
        ExprCrossover { alters }
    }
}

impl Crossover<AnyChromosome<'static>> for ExprCrossover {
    fn name(&self) -> String {
        "ExpressionCrossover".into()
    }

    fn cross_chromosomes(
        &self,
        chrom_one: &mut AnyChromosome<'static>,
        chrom_two: &mut AnyChromosome<'static>,
        _: f32,
    ) -> AlterResult {
        let mut count = 0;

        let n = std::cmp::min(chrom_one.len(), chrom_two.len());

        if n == 0 {
            return count.into();
        }

        for alter in &self.alters {
            match alter.expr {
                Expr::Cross(CrossoverExpr::OnePoint) => {
                    let cut = if n == 1 {
                        0
                    } else {
                        random_provider::range(0..n)
                    };

                    for i in cut..n {
                        let a = chrom_one.genes_mut()[i].allele_mut();
                        let b = chrom_two.genes_mut()[i].allele_mut();

                        count += Expr::Cross(CrossoverExpr::Swap).apply_pair(a, b);
                    }
                }
                Expr::Cross(CrossoverExpr::TwoPoint) => {
                    if n < 2 || random_provider::random::<f32>() > alter.p {
                        continue;
                    }

                    let i = random_provider::range(0..(n - 1));
                    let j = random_provider::range((i + 1)..n);

                    for k in i..j {
                        let mut a = &mut chrom_one.genes_mut()[0..2]; //.allele_mut();
                        let mut b = &mut chrom_two.genes_mut()[0..2]; //.allele_mut();

                        std::mem::swap(&mut a, &mut b);

                        // count += Expr::Cross(CrossoverExpr::Swap).eval_on_pair(a, b);
                    }
                }
                Expr::Cross(CrossoverExpr::Swap) | Expr::Cross(CrossoverExpr::Mean) => {
                    for idx in 0..n {
                        if random_provider::random::<f32>() < alter.p {
                            let a = chrom_one.genes_mut()[idx].allele_mut();
                            let b = chrom_two.genes_mut()[idx].allele_mut();

                            // count += alter.expr.apply_pair(a, b);
                        }
                    }
                }
                _ => {}
            }
        }

        count.into()
    }
}

// fn apply_single_into<'a>(
//     target: &str,
//     dtype: &DataType,
//     p: f32,
//     expr: &MutateExpr,
//     value: &mut AnyValue<'a>,
// ) -> bool {
//     let mut changed = false;

//     match value {
//         AnyValue::Struct(_) => {
//             if let Some(field) = value.get_struct_value_mut(target) {
//                 changed |= apply_single_into(target, dtype, p, expr, field);
//             } else if let AnyValue::Struct(pairs) = value {
//                 for (v, _) in pairs.iter_mut() {
//                     changed |= apply_single_into(target, dtype, p, expr, v);
//                 }
//             }

//             changed
//         }
//         AnyValue::Vector(vec) => {
//             for v in vec.iter_mut() {
//                 changed |= apply_single_into(target, dtype, p, expr, v);
//             }

//             changed
//         }
//         _ => {
//             if value.dtype() != *dtype {
//                 return false;
//             }

//             match &expr {
//                 MutateExpr::Uniform(amount) => {
//                     value.with_numeric_mut(|slot| {
//                         crate::value::apply_numeric_slot_mut(
//                             slot,
//                             |x_f32| {
//                                 let delta = random_provider::range(amount.clone());
//                                 x_f32 + delta
//                             },
//                             |x_f64| {
//                                 let delta = random_provider::range(amount.clone()) as f64;
//                                 x_f64 + delta
//                             },
//                             |i, unsigned| {
//                                 // Sample f32 delta from range, round to nearest int
//                                 let delta = random_provider::range(amount.clone()).round() as i128;
//                                 let y = i.saturating_add(delta);
//                                 if unsigned { y.max(0) } else { y }
//                             },
//                         );

//                         changed = true;
//                     });
//                 }
//                 MutateExpr::Gaussian(mean, stddev) => {
//                     let mu = *mean as f64;
//                     let sd = (*stddev as f64).max(1e-12);
//                     value.with_numeric_mut(|slot| {
//                         crate::value::apply_numeric_slot_mut(
//                             slot,
//                             |x_f32| {
//                                 let delta = random_provider::gaussian(mu, sd) as f32;
//                                 x_f32 + delta
//                             },
//                             |x_f64| {
//                                 let delta = random_provider::gaussian(mu, sd);
//                                 x_f64 + delta
//                             },
//                             |i, unsigned| {
//                                 // Integer: gaussian delta rounded to nearest int
//                                 let delta = random_provider::gaussian(mu, sd).round() as i128;
//                                 let y = i.saturating_add(delta);
//                                 if unsigned { y.max(0) } else { y }
//                             },
//                         );

//                         changed = true;
//                     });
//                 }
//                 MutateExpr::Jitter(fraction) => {
//                     let frac = *fraction as f64;
//                     value.with_numeric_mut(|slot| {
//                         crate::value::apply_numeric_slot_mut(
//                             slot,
//                             |x_f32| {
//                                 let change = random_provider::range(-1.0..1.0) * frac;
//                                 x_f32 + change as f32
//                             },
//                             |x_f64| {
//                                 let change = random_provider::gaussian(0.0, frac * x_f64.abs());
//                                 x_f64 + change
//                             },
//                             |i, unsigned| {
//                                 // Integer: gaussian change rounded to nearest int
//                                 let change = random_provider::gaussian(0.0, frac * (i.abs() as f64))
//                                     .round() as i128;
//                                 let y = i.saturating_add(change);
//                                 if unsigned { y.max(0) } else { y }
//                             },
//                         );

//                         changed = true;
//                     });
//                 }
//             }

//             changed
//         }
//     }
// }

// fn apply_pair_into<'a>(
//     target: &str,
//     dtype: &DataType,
//     p: f32,
//     expr: &CrossoverExpr,
//     a: &mut AnyValue<'a>,
//     b: &mut AnyValue<'a>,
// ) -> bool {
//     use AnyValue::*;
//     let mut changed = false;

//     match (a, b) {
//         one @ (Struct(_), Struct(_)) => {
//             if let (Some(field), Some(field2)) = (
//                 one.0.get_struct_value_mut(target),
//                 one.1.get_struct_value_mut(target),
//             ) {
//                 changed |= apply_pair_into(target, dtype, p, expr, field, field2);
//             } else if let (Struct(sa), Struct(sb)) = (one.0, one.1) {
//                 let n = sa.len().min(sb.len());
//                 for i in 0..n {
//                     let (va, _) = &mut sa[i];
//                     let (vb, _) = &mut sb[i];
//                     changed |= apply_pair_into(target, dtype, p, expr, va, vb);
//                 }
//             }

//             return changed;
//         }
//         (Vector(va), Vector(vb)) => {
//             let n = va.len().min(vb.len());
//             for i in 0..n {
//                 changed |= apply_pair_into(target, dtype, p, expr, &mut va[i], &mut vb[i]);
//             }
//         }
//         (la, lb) => {
//             if la.dtype() != *dtype || lb.dtype() != *dtype {
//                 return false;
//             }

//             let did = match expr {
//                 CrossoverExpr::Swap => swap_leaf(la, lb),
//                 CrossoverExpr::Mean => mean_leaf(la, lb),
//                 // OnePoint/TwoPoint are handled at the gene/segment level, not at leaves.
//                 CrossoverExpr::OnePoint | CrossoverExpr::TwoPoint => false,
//             };
//             changed |= did;
//         }
//     }

//     changed
// }
