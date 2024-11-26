use crate::{random_provider, Chromosome, Crossover, FloatGene, Gene};

pub struct IntermediateCrossover {
    rate: f32,
    alpha: f32,
}

impl IntermediateCrossover {
    pub fn new(rate: f32, alpha: f32) -> Self {
        IntermediateCrossover { rate, alpha }
    }
}

impl<C: Chromosome<GeneType = FloatGene>> Crossover<C> for IntermediateCrossover {
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "IntermediateCrossover"
    }

    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C) -> i32 {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::random::<f32>() < self.rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                let allele1 = gene_one.allele;
                let allele2 = gene_two.allele;

                let alpha = random_provider::gen_range(0.0..self.alpha);
                let allele = allele1 * alpha + allele2 * (1.0 - alpha);

                let new_gene = gene_one.from_allele(&allele);

                chrom_one.set_gene(i, new_gene);
                cross_count += 1;
            }
        }

        cross_count
    }
}
