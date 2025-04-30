use radiate_core::{AlterResult, Chromosome, Crossover, random_provider};

pub struct ShuffleCrossover {
    rate: f32,
}

impl ShuffleCrossover {
    pub fn new(rate: f32) -> Self {
        ShuffleCrossover { rate }
    }
}

impl<C: Chromosome> Crossover<C> for ShuffleCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, _: f32) -> AlterResult {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());
        if length < 2 {
            return 0.into();
        }

        let mut indices: Vec<usize> = (0..length).collect();
        random_provider::shuffle(&mut indices);

        let temp_chrom_one = chrom_one.clone();
        let temp_chrom_two = chrom_two.clone();

        let mut cross_count = 0;
        for (i, &index) in indices.iter().enumerate() {
            if i % 2 == 0 {
                chrom_one.set(index, temp_chrom_two.get(index).clone());
                chrom_two.set(index, temp_chrom_one.get(index).clone());
                cross_count += 1;
            }
        }

        cross_count.into()
    }
}
