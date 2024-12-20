use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome};

pub struct ShuffleCrossover {
    rate: f32,
}

impl ShuffleCrossover {
    pub fn new(rate: f32) -> Self {
        ShuffleCrossover { rate }
    }
}

impl<C: Chromosome> Alter<C> for ShuffleCrossover {
    fn name(&self) -> &'static str {
        "ShuffleCrossover"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C) -> i32 {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());
        if length < 2 {
            return 0;
        }

        let mut indices: Vec<usize> = (0..length).collect();
        random_provider::shuffle(&mut indices);

        let temp_chrom_one = chrom_one.clone();
        let temp_chrom_two = chrom_two.clone();

        for (i, &index) in indices.iter().enumerate() {
            if i % 2 == 0 {
                chrom_one.set_gene(index, temp_chrom_two.get_gene(index).clone());
                chrom_two.set_gene(index, temp_chrom_one.get_gene(index).clone());
            }
        }

        1
    }
}
