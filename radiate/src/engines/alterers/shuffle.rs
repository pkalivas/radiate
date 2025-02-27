use super::{Alter, AlterAction, Alterer, Crossover, IntoAlter};
use crate::{Chromosome, EngineCompoment, random_provider};

pub struct ShuffleCrossover {
    rate: f32,
}

impl ShuffleCrossover {
    pub fn new(rate: f32) -> Self {
        ShuffleCrossover { rate }
    }
}

impl EngineCompoment for ShuffleCrossover {
    fn name(&self) -> &'static str {
        "ShuffleCrossover"
    }
}

impl<C: Chromosome> Alter<C> for ShuffleCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<C: Chromosome> Crossover<C> for ShuffleCrossover {
    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, _: f32) -> i32 {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());
        if length < 2 {
            return 0;
        }

        let mut indices: Vec<usize> = (0..length).collect();
        random_provider::shuffle(&mut indices);

        let temp_chrom_one = chrom_one.clone();
        let temp_chrom_two = chrom_two.clone();

        let mut cross_count = 0;
        for (i, &index) in indices.iter().enumerate() {
            if i % 2 == 0 {
                chrom_one.set_gene(index, temp_chrom_two.get_gene(index).clone());
                chrom_two.set_gene(index, temp_chrom_one.get_gene(index).clone());
                cross_count += 1;
            }
        }

        cross_count
    }
}

impl<C: Chromosome> IntoAlter<C> for ShuffleCrossover {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "ShuffleCrossover",
            self.rate,
            AlterAction::Crossover(Box::new(self)),
        )
    }
}
