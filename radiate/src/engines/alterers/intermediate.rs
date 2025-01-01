use crate::alter::AlterType;
use crate::{random_provider, Alter, Chromosome, FloatGene, Gene};

/// Intermediate Crossover. This crossover method takes two chromosomes and crosses them
/// by taking a weighted average of the two alleles. The weight is determined by the `alpha`
/// parameter. The new allele is calculated as:
/// ```text
/// a = a1 * alpha + a2 * (1 - alpha)
/// ```
/// where `a` is the new allele, `a1` is the allele from the first chromosome, `a2` is the allele
/// from the second chromosome, and `alpha` is a value between 0 and 1.
///
pub struct IntermediateCrossover {
    rate: f32,
    alpha: f32,
}

impl IntermediateCrossover {
    pub fn new(rate: f32, alpha: f32) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("Rate must be between 0 and 1");
        }

        if alpha < 0.0 || alpha > 1.0 {
            panic!("Alpha must be between 0 and 1");
        }

        IntermediateCrossover { rate, alpha }
    }
}

impl<C: Chromosome<Gene = FloatGene>> Alter<C> for IntermediateCrossover {
    fn name(&self) -> &'static str {
        "IntermediateCrossover"
    }
    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
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

                let new_gene = gene_one.with_allele(&allele);

                chrom_one.set_gene(i, new_gene);
                cross_count += 1;
            }
        }

        cross_count
    }
}