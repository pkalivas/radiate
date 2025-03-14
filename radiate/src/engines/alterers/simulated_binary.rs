use super::{AlterAction, AlterResult, Crossover, IntoAlter};
use crate::{ArithmeticGene, Chromosome, FloatGene, Gene, random_provider};

pub struct SimulatedBinaryCrossover {
    contiguty: f32,
    crossover_rate: f32,
}

impl SimulatedBinaryCrossover {
    pub fn new(contiguty: f32, crossover_rate: f32) -> Self {
        Self {
            contiguty,
            crossover_rate,
        }
    }
}

impl<C: Chromosome<Gene = FloatGene>> Crossover<C> for SimulatedBinaryCrossover {
    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, _: f32) -> AlterResult {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return 0.into();
        }

        let mut count = 0;

        for i in 0..length {
            if random_provider::range(0..2) == 0 {
                let u = random_provider::random::<f32>();
                let beta = if u <= 0.5 {
                    (2.0 * u).powf(1.0 / (self.contiguty + 1.0))
                } else {
                    (0.5 / (1.0 - u)).powf(1.0 / (self.contiguty + 1.0))
                };

                let v1 = chrom_one.get_gene(i).allele();
                let v2 = chrom_two.get_gene(i).allele();

                let v = if random_provider::range(0..2) == 0 {
                    (v1 - v2) * 0.5 - (beta * 0.5 * (v1 - v2).abs())
                } else {
                    (v1 - v2) * 0.5 + (beta * 0.5 * (v1 - v2).abs())
                };

                let new_gene = v.clamp(*chrom_one.get_gene(i).min(), *chrom_one.get_gene(i).max());

                count += 1;

                chrom_one.set_gene(i, chrom_one.get_gene(i).with_allele(&new_gene));
            }
        }

        count.into()
    }
}

impl<C: Chromosome<Gene = FloatGene>> IntoAlter<C> for SimulatedBinaryCrossover {
    fn into_alter(self) -> super::Alterer<C> {
        super::Alterer::new(
            "SimulatedBinaryCrossover",
            self.crossover_rate,
            AlterAction::Crossover(Box::new(self)),
        )
    }
}
