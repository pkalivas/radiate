use super::{Alter, AlterAction, Crossover, EngineCompoment};
use crate::{random_provider, Chromosome, FloatGene, Gene, NumericGene};

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

    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}

impl EngineCompoment for SimulatedBinaryCrossover {
    fn name(&self) -> &'static str {
        "Simulated Binary Crossover"
    }
}

impl<C: Chromosome<Gene = FloatGene>> Alter<C> for SimulatedBinaryCrossover {
    fn rate(&self) -> f32 {
        self.crossover_rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Crossover(Box::new(self))
    }
}

impl<C: Chromosome<Gene = FloatGene>> Crossover<C> for SimulatedBinaryCrossover {
    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C) -> i32 {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return 0;
        }

        let mut count = 0;

        for i in 0..length {
            if random_provider::gen_range(0..2) == 0 {
                let u = random_provider::random::<f32>();
                let beta = if u <= 0.5 {
                    (2.0 * u).powf(1.0 / (self.contiguty + 1.0))
                } else {
                    (0.5 / (1.0 - u)).powf(1.0 / (self.contiguty + 1.0))
                };

                let v1 = chrom_one.get_gene(i).allele();
                let v2 = chrom_two.get_gene(i).allele();

                let v = if random_provider::gen_range(0..2) == 0 {
                    (v1 - v2) * 0.5 - (beta * 0.5 * (v1 - v2).abs())
                } else {
                    (v1 - v2) * 0.5 + (beta * 0.5 * (v1 - v2).abs())
                };

                let new_gene = SimulatedBinaryCrossover::clamp(
                    v,
                    *chrom_one.get_gene(i).min(),
                    *chrom_one.get_gene(i).max(),
                );

                count += 1;

                chrom_one.set_gene(i, chrom_one.get_gene(i).with_allele(&new_gene));
            }
        }

        count
    }
}
