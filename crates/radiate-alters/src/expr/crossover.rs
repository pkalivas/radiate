use core::fmt;
use radiate_core::{
    chromosomes::gene::{HasNumericSlot, NumericSlotMut, apply_pair_numeric_slot_mut},
    random_provider,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Xover<G> {
    op: Arc<dyn CrossOp<G> + Send + Sync + 'static>,
    name: Option<&'static str>,
}

impl<G> Xover<G> {
    pub fn new<F>(op: F) -> Self
    where
        F: CrossOp<G> + Send + Sync + 'static,
    {
        Self {
            op: Arc::new(op),
            name: None,
        }
    }

    pub fn named<F>(name: &'static str, op: F) -> Self
    where
        F: CrossOp<G> + Send + Sync + 'static,
    {
        Self {
            op: Arc::new(op),
            name: Some(name),
        }
    }

    pub fn name(&self) -> Option<&'static str> {
        self.name
    }

    #[inline]
    pub fn op(&self) -> &dyn CrossOp<G> {
        &*self.op
    }
}

impl<G> fmt::Debug for Xover<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.name {
            write!(f, "Xover({n})")
        } else {
            write!(f, "Xover(<op>)")
        }
    }
}

#[derive(Clone)]
pub enum CrossoverExpr {
    OnePoint,
    TwoPoint,
    Swap,
}

#[derive(Clone)]
pub enum NumericCrossoverExpr {
    Blend(f32),
    Intermediate(f32),
    Mean,
}

impl<G> Into<Xover<G>> for CrossoverExpr {
    fn into(self) -> Xover<G> {
        match self {
            CrossoverExpr::OnePoint => Xover::new(CrossoverExpr::OnePoint),
            CrossoverExpr::TwoPoint => Xover::new(CrossoverExpr::TwoPoint),
            CrossoverExpr::Swap => Xover::new(CrossoverExpr::Swap),
        }
    }
}

impl<G: HasNumericSlot> Into<Xover<G>> for NumericCrossoverExpr {
    fn into(self) -> Xover<G> {
        match self {
            NumericCrossoverExpr::Blend(factor) => Xover::new(NumericCrossoverExpr::Blend(factor)),
            NumericCrossoverExpr::Intermediate(factor) => {
                Xover::new(NumericCrossoverExpr::Intermediate(factor))
            }
            NumericCrossoverExpr::Mean => Xover::new(NumericCrossoverExpr::Mean),
        }
    }
}

pub trait CrossOp<G> {
    fn apply_on_slices(&self, a: &mut [G], b: &mut [G]) -> usize;
    fn apply_on_pair(&self, a: &mut G, b: &mut G) -> usize;
}

impl<G: HasNumericSlot> CrossOp<G> for NumericCrossoverExpr {
    fn apply_on_slices(&self, one: &mut [G], two: &mut [G]) -> usize {
        match self {
            NumericCrossoverExpr::Blend(factor) => {
                let mut count = 0;
                for (a_gene, b_gene) in one.iter_mut().zip(two.iter_mut()) {
                    count += blend_crossover(a_gene, b_gene, *factor);
                }
                count
            }
            NumericCrossoverExpr::Mean => {
                let mut count = 0;
                for (a_gene, b_gene) in one.iter_mut().zip(two.iter_mut()) {
                    count += crossover_mean(a_gene, b_gene);
                }
                count
            }
            _ => 0,
        }
    }

    fn apply_on_pair(&self, one: &mut G, two: &mut G) -> usize {
        match self {
            NumericCrossoverExpr::Blend(factor) => blend_crossover(one, two, *factor),
            NumericCrossoverExpr::Mean => crossover_mean(one, two),
            NumericCrossoverExpr::Intermediate(factor) => intermediate_crossover(one, two, *factor),
        }
    }
}

impl<G> CrossOp<G> for CrossoverExpr {
    fn apply_on_slices(&self, one: &mut [G], two: &mut [G]) -> usize {
        match self {
            CrossoverExpr::OnePoint => single_point_crossover(one, two),
            CrossoverExpr::TwoPoint => multi_point_crossover(one, two, 2),
            CrossoverExpr::Swap => {
                let n = one.len().min(two.len());

                for i in 0..n {
                    if random_provider::bool(0.5) {
                        std::mem::swap(&mut one[i], &mut two[i]);
                    }
                }

                n
            }
        }
    }

    fn apply_on_pair(&self, a: &mut G, b: &mut G) -> usize {
        match self {
            CrossoverExpr::Swap => {
                std::mem::swap(a, b);
                1
            }
            _ => 0,
        }
    }
}

#[inline]
fn intermediate_crossover<N: HasNumericSlot>(a: &mut N, b: &mut N, alpha: f32) -> usize {
    if let (Some(a_slot), Some(b_slot)) = (a.numeric_slot_mut(), b.numeric_slot_mut()) {
        apply_pair_numeric_slot_mut(
            a_slot,
            b_slot,
            |x1, x2| {
                let new_x1 = x1 * alpha + x2 * (1.0 - alpha);
                let new_x2 = x2 * alpha + x1 * (1.0 - alpha);
                (new_x1, new_x2)
            },
            |x1, x2| {
                let new_x1 = x1 * (alpha as f64) + x2 * (1.0 - alpha as f64);
                let new_x2 = x2 * (alpha as f64) + x1 * (1.0 - alpha as f64);
                (new_x1, new_x2)
            },
            |i1, i2, unsigned| {
                let new_i1 = (i1 as f32 * alpha + i2 as f32 * (1.0 - alpha)).round() as i128;
                let new_i2 = (i2 as f32 * alpha + i1 as f32 * (1.0 - alpha)).round() as i128;
                if unsigned {
                    (new_i1.max(0), new_i2.max(0))
                } else {
                    (new_i1, new_i2)
                }
            },
        );

        return 1;
    }
    0
}

#[inline]
pub fn multi_point_crossover<G>(
    chrom_one: &mut [G],
    chrom_two: &mut [G],
    num_points: usize,
) -> usize {
    let length = std::cmp::min(chrom_one.len(), chrom_two.len());

    if length < 2 {
        return 0;
    }

    let mut crossover_points = random_provider::indexes(0..length);
    random_provider::shuffle(&mut crossover_points);

    let selected_points = &mut crossover_points[..num_points];
    selected_points.sort();

    let mut current_parent = 1;
    let mut last_point = 0;

    for i in selected_points {
        if current_parent == 1 {
            chrom_one[last_point..*i].swap_with_slice(&mut chrom_two[last_point..*i]);
        }

        current_parent = 3 - current_parent;
        last_point = *i;
    }

    if current_parent == 1 {
        chrom_one[last_point..].swap_with_slice(&mut chrom_two[last_point..]);
    }

    crossover_points.len()
}

#[inline]
pub fn single_point_crossover<G>(chrom_one: &mut [G], chrom_two: &mut [G]) -> usize {
    let length = std::cmp::min(chrom_one.len(), chrom_two.len());

    if length < 2 {
        return 0;
    }

    let crossover_point = random_provider::range(1..length);
    chrom_one[crossover_point..].swap_with_slice(&mut chrom_two[crossover_point..]);

    1
}

#[inline]
fn blend_crossover<N: HasNumericSlot>(one: &mut N, two: &mut N, alpha: f32) -> usize {
    let slot_one = one.numeric_slot_mut();
    let slot_two = two.numeric_slot_mut();

    if let (Some(slot_one), Some(slot_two)) = (slot_one, slot_two) {
        let a = alpha as f64;
        apply_pair_numeric_slot_mut(
            slot_one,
            slot_two,
            |v1, v2| (v1 - (alpha * (v2 - v1)), v2 - (alpha * (v1 - v2))),
            |f64_1, f64_2| (f64_1 - (a * (f64_2 - f64_1)), f64_2 - (a * (f64_1 - f64_2))),
            |_, _, _| panic!("i32 blending not supported"),
        );

        return 1;
    }

    0
}

#[inline]
fn crossover_mean<N: HasNumericSlot>(a: &mut N, b: &mut N) -> usize {
    if let (Some(a), Some(b)) = (a.numeric_slot_mut(), b.numeric_slot_mut()) {
        let mutated = match (a, b) {
            (NumericSlotMut::F32(aa), NumericSlotMut::F32(bb)) => {
                let m = (*aa + *bb) * 0.5;
                *aa = m;
                true
            }
            (NumericSlotMut::F64(aa), NumericSlotMut::F64(bb)) => {
                let m = (*aa + *bb) * 0.5;
                *aa = m;
                true
            }
            (NumericSlotMut::I8(aa), NumericSlotMut::I8(bb)) => {
                let m = ((*aa as i32 + *bb as i32) / 2) as i8;
                *aa = m;
                true
            }
            (NumericSlotMut::I16(aa), NumericSlotMut::I16(bb)) => {
                let m = ((*aa as i32 + *bb as i32) / 2) as i16;
                *aa = m;
                true
            }
            (NumericSlotMut::I32(aa), NumericSlotMut::I32(bb)) => {
                let m = ((*aa as i64 + *bb as i64) / 2) as i32;
                *aa = m;
                true
            }
            (NumericSlotMut::I64(aa), NumericSlotMut::I64(bb)) => {
                let m = ((*aa as i128 + *bb as i128) / 2) as i64;
                *aa = m;
                true
            }
            (NumericSlotMut::U8(aa), NumericSlotMut::U8(bb)) => {
                let m = ((*aa as u32 + *bb as u32) / 2) as u8;
                *aa = m;
                true
            }
            (NumericSlotMut::U16(aa), NumericSlotMut::U16(bb)) => {
                let m = ((*aa as u32 + *bb as u32) / 2) as u16;
                *aa = m;
                true
            }
            (NumericSlotMut::U32(aa), NumericSlotMut::U32(bb)) => {
                let m = ((*aa as u64 + *bb as u64) / 2) as u32;
                *aa = m;
                true
            }
            (NumericSlotMut::U64(aa), NumericSlotMut::U64(bb)) => {
                let m = (*aa >> 1) + (*bb >> 1) + ((*aa & *bb) & 1);
                *aa = m;
                true
            }
            _ => false,
        };

        if mutated {
            return 1;
        }
    }

    0
}
