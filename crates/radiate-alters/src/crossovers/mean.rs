use radiate_core::{
    AlterResult, ArithmeticGene, Chromosome, Crossover,
    chromosomes::gene::{HasNumericSlot, NumericSlotMut},
    random_provider,
};

/// The `MeanCrossover` is a simple crossover method that replaces the genes of the first chromosome
/// with the mean of the two genes. The mean is calculated by adding the two genes together and dividing
/// by two.
///
/// This crossover can only be used with `ArithmeticGene`s and can be largely benifitial. However, keep
/// in mind that because we are taking the mean of two genes, this results in children that
/// converge towards a common distribution. This can be useful in some cases, but it can also
/// result in a loss of diversity in the population in others.
pub struct MeanCrossover {
    rate: f32,
}

impl MeanCrossover {
    /// Create a new instance of the `MeanCrossover` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("The rate must be between 0.0 and 1.0");
        }

        MeanCrossover { rate }
    }
}

impl<C: Chromosome> Crossover<C> for MeanCrossover
where
    C::Gene: ArithmeticGene,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;

        for (gene_one, gene_two) in chrom_one.iter_mut().zip(chrom_two.iter()) {
            if random_provider::random::<f32>() < rate {
                *gene_one = gene_one.mean(gene_two);
                count += 1;
            }
        }

        count.into()
    }
}

pub fn crossover_mean<N: HasNumericSlot>(a: &mut N, b: &mut N) -> usize {
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
