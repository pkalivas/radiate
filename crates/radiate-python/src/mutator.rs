use crate::{AnyChromosome, AnyValue, DataType};
use pyo3::{pyclass, pymethods};
use radiate::{AlterResult, Chromosome, Crossover, Mutate, random_provider};
use std::ops::Range;

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
        let count = 0;

        for gene in chromosome.iter_mut() {
            for alteration in &self.alters {
                alteration.apply_single(gene.allele_mut());
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
        rate: f32,
    ) -> AlterResult {
        let mut count = 0;

        let n = std::cmp::min(chrom_one.len(), chrom_two.len());

        if n == 0 {
            return count.into();
        }

        for alter in &self.alters {
            match &alter.expr {
                // Segment crossovers across genes
                AlterExpr::Crossover(CrossoverExpr::OnePoint) => {
                    if random_provider::random::<f32>() >= rate {
                        continue;
                    }

                    let cut = if n == 1 {
                        0
                    } else {
                        random_provider::range(0..n)
                    };

                    let mut changed_here = 0;
                    for i in cut..n {
                        let a = chrom_one.genes_mut()[i].allele_mut();
                        let b = chrom_two.genes_mut()[i].allele_mut();

                        changed_here += alter.apply_pair(a, b) as usize;
                    }

                    if changed_here > 0 {
                        count += 1;
                    }
                }
                AlterExpr::Crossover(CrossoverExpr::TwoPoint) => {
                    if n < 2 || random_provider::random::<f32>() >= rate {
                        continue;
                    }

                    let i = random_provider::range(0..(n - 1));
                    let j = random_provider::range((i + 1)..n);

                    let mut changed_here = 0;
                    for k in i..j {
                        let a = chrom_one.genes_mut()[k].allele_mut();
                        let b = chrom_two.genes_mut()[k].allele_mut();

                        changed_here += alter.apply_pair(a, b) as usize;
                    }

                    if changed_here > 0 {
                        count += 1;
                    }
                }
                // Per-gene pairwise crossovers (swap/mean) under per-gene rate
                AlterExpr::Crossover(CrossoverExpr::Swap)
                | AlterExpr::Crossover(CrossoverExpr::Mean) => {
                    for idx in 0..n {
                        if random_provider::random::<f32>() >= rate {
                            continue;
                        }

                        let a = chrom_one.genes_mut()[idx].allele_mut();
                        let b = chrom_two.genes_mut()[idx].allele_mut();

                        if alter.apply_pair(a, b) {
                            count += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        count.into()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyAlteration {
    pub inner: Alteration,
}

#[pymethods]
impl PyAlteration {
    #[staticmethod]
    #[pyo3(signature = (range, target, dtype, p=1.0))]
    fn uniform(range: (f32, f32), target: String, dtype: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "uniform".into(),
                AlterExpr::Mutate(MutateExpr::Uniform(range.0..range.1)),
                target,
                DataType::from(dtype),
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (mean, stddev, target, dtype, p=1.0))]
    fn gaussian(mean: f32, stddev: f32, target: String, dtype: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "gaussian".into(),
                AlterExpr::Mutate(MutateExpr::Gaussian(mean, stddev)),
                target,
                DataType::from(dtype),
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (amount, target, dtype, p=1.0))]
    fn jitter(amount: f32, target: String, dtype: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "jitter".into(),
                AlterExpr::Mutate(MutateExpr::Jitter(amount)),
                target,
                DataType::from(dtype),
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, dtype, p=1.0))]
    fn swap(target: String, dtype: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "swap".into(),
                AlterExpr::Crossover(CrossoverExpr::Swap),
                target,
                DataType::from(dtype),
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, dtype, p=1.0))]
    fn mean(target: String, dtype: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "mean".into(),
                AlterExpr::Crossover(CrossoverExpr::Mean),
                target,
                DataType::from(dtype),
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, dtype, p=1.0))]
    fn one_point(target: String, dtype: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "one_point".into(),
                AlterExpr::Crossover(CrossoverExpr::OnePoint),
                target,
                DataType::from(dtype),
                p,
            ),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (target, dtype, p=1.0))]
    fn two_point(target: String, dtype: String, p: f32) -> Self {
        PyAlteration {
            inner: Alteration::new(
                "two_point".into(),
                AlterExpr::Crossover(CrossoverExpr::TwoPoint),
                target,
                DataType::from(dtype),
                p,
            ),
        }
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
    Swap,
    OnePoint,
    TwoPoint,
    Mean,
}

#[derive(Debug, Clone)]
pub enum AlterExpr {
    Mutate(MutateExpr),
    Crossover(CrossoverExpr),
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Alteration {
    pub name: String,
    pub expr: AlterExpr,
    pub target: String,
    pub dtype: DataType,
    pub p: f32,
}

impl Alteration {
    pub fn new(name: String, expr: AlterExpr, target: String, dtype: DataType, p: f32) -> Self {
        Alteration {
            name,
            expr,
            target,
            dtype,
            p,
        }
    }

    pub fn apply_single<'a>(&self, value: &mut AnyValue<'a>) -> bool {
        apply_single_into(
            &self.target,
            &self.dtype,
            self.p,
            match &self.expr {
                AlterExpr::Mutate(m) => m,
                _ => return false,
            },
            value,
        );

        true
    }

    pub fn apply_pair<'a>(&self, a: &mut AnyValue<'a>, b: &mut AnyValue<'a>) -> bool {
        apply_pair_into(
            &self.target,
            &self.dtype,
            self.p,
            match &self.expr {
                AlterExpr::Crossover(c) => c,
                _ => return false,
            },
            a,
            b,
        )
    }
}

fn apply_single_into<'a>(
    target: &str,
    dtype: &DataType,
    p: f32,
    expr: &MutateExpr,
    value: &mut AnyValue<'a>,
) {
    if p <= 1.0 && random_provider::random::<f32>() > p {
        return;
    }

    match value {
        AnyValue::Struct(_) => {
            if let Some(field) = value.get_struct_value_mut(target) {
                apply_single_into(target, dtype, p, expr, field);
            } else if let AnyValue::Struct(pairs) = value {
                for (v, _) in pairs.iter_mut() {
                    apply_single_into(target, dtype, p, expr, v);
                }
            }
        }
        AnyValue::Vector(vec) => {
            for v in vec.iter_mut() {
                apply_single_into(target, dtype, p, expr, v);
            }
        }
        _ => {
            if value.dtype() != *dtype {
                return;
            }

            match &expr {
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
                    });
                }
                MutateExpr::Jitter(fraction) => {
                    let frac = *fraction as f64;
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
                    });
                }
            }
        }
    }
}

fn apply_pair_into<'a>(
    target: &str,
    dtype: &DataType,
    p: f32,
    expr: &CrossoverExpr,
    a: &mut AnyValue<'a>,
    b: &mut AnyValue<'a>,
) -> bool {
    use AnyValue::*;
    let mut changed = false;

    match (a, b) {
        (Struct(sa), Struct(sb)) => {
            let mut had_direct = false;
            let len = sa.len().min(sb.len());
            for i in 0..len {
                let (va, fa) = &mut sa[i];
                let (vb, fb) = &mut sb[i];
                debug_assert_eq!(fa.name(), fb.name());
                if fa.name() == target {
                    had_direct = true;
                    changed |= apply_pair_into(target, dtype, p, expr, va, vb);
                }
            }
            if !had_direct {
                for i in 0..len {
                    let (va, _) = &mut sa[i];
                    let (vb, _) = &mut sb[i];
                    match (&va, &vb) {
                        (Struct(_), Struct(_)) | (Vector(_), Vector(_)) => {
                            changed |= apply_pair_into(target, dtype, p, expr, va, vb);
                        }
                        _ => {}
                    }
                }
            }
        }
        (Vector(va), Vector(vb)) => {
            let n = va.len().min(vb.len());
            for i in 0..n {
                changed |= apply_pair_into(target, dtype, p, expr, &mut va[i], &mut vb[i]);
            }
        }
        (la, lb) => {
            if la.dtype() != *dtype || lb.dtype() != *dtype {
                return false;
            }

            if p < 1.0 && random_provider::random::<f32>() > p {
                return false;
            }

            let did = match expr {
                CrossoverExpr::Swap => swap_leaf(la, lb),
                CrossoverExpr::Mean => mean_leaf(la, lb),
                // OnePoint/TwoPoint are handled at the gene/segment level, not at leaves.
                CrossoverExpr::OnePoint | CrossoverExpr::TwoPoint => false,
            };
            changed |= did;
        }
    }

    changed
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let alteration = Alteration::new(
            "test".into(),
            AlterExpr::Mutate(MutateExpr::Uniform(-100.0..100.0)),
            "target".into(),
            DataType::Float32,
            0.99,
        );

        let gaussian_alteration = Alteration::new(
            "test".into(),
            AlterExpr::Mutate(MutateExpr::Jitter(0.1)),
            "target".into(),
            DataType::Float32,
            1.0,
        );

        let mut value = AnyValue::Float32(1.0);

        alteration.apply_single(&mut value);
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
        gaussian_alteration.apply_single(&mut nested_value);
        println!("Nested value after alteration: {:?}", nested_value);
    }
}
