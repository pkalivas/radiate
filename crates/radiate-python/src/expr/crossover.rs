use crate::{CrossoverExpr, ExprNode, ExprValue, NumericSlotMut};
use radiate::random_provider;

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
                radiate::crossovers::multipoint::crossover_single_point(a, b)
            }

            CrossoverExpr::TwoPoint => {
                radiate::crossovers::multipoint::crossover_multi_point(a, b, 2)
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

fn set_both_mean(a: &mut NumericSlotMut<'_>, b: &mut NumericSlotMut<'_>) -> bool {
    match (a, b) {
        (NumericSlotMut::F32(aa), NumericSlotMut::F32(bb)) => {
            let m = (**aa + **bb) * 0.5;
            **aa = m;
            true
        }
        (NumericSlotMut::F64(aa), NumericSlotMut::F64(bb)) => {
            let m = (**aa + **bb) * 0.5;
            **aa = m;
            true
        }
        (NumericSlotMut::I8(aa), NumericSlotMut::I8(bb)) => {
            let m = ((**aa as i32 + **bb as i32) / 2) as i8;
            **aa = m;
            true
        }
        (NumericSlotMut::I16(aa), NumericSlotMut::I16(bb)) => {
            let m = ((**aa as i32 + **bb as i32) / 2) as i16;
            **aa = m;
            true
        }
        (NumericSlotMut::I32(aa), NumericSlotMut::I32(bb)) => {
            let m = ((**aa as i64 + **bb as i64) / 2) as i32;
            **aa = m;
            true
        }
        (NumericSlotMut::I64(aa), NumericSlotMut::I64(bb)) => {
            let m = ((**aa as i128 + **bb as i128) / 2) as i64;
            **aa = m;
            true
        }
        (NumericSlotMut::U8(aa), NumericSlotMut::U8(bb)) => {
            let m = ((**aa as u32 + **bb as u32) / 2) as u8;
            **aa = m;
            true
        }
        (NumericSlotMut::U16(aa), NumericSlotMut::U16(bb)) => {
            let m = ((**aa as u32 + **bb as u32) / 2) as u16;
            **aa = m;
            true
        }
        (NumericSlotMut::U32(aa), NumericSlotMut::U32(bb)) => {
            let m = ((**aa as u64 + **bb as u64) / 2) as u32;
            **aa = m;
            true
        }
        (NumericSlotMut::U64(aa), NumericSlotMut::U64(bb)) => {
            let m = (**aa >> 1) + (**bb >> 1) + ((**aa & **bb) & 1);
            **aa = m;
            true
        }
        _ => false,
    }
}
