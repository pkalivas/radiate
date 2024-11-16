use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::genotype::Genotype;
use crate::engines::genome::phenotype::Phenotype;
use crate::engines::genome::population::Population;

pub trait Crossover<G, A>
where
    G: Gene<G, A>,
{
    fn cross_rate(&self) -> f32;

    #[inline]
    fn cross(&self, population: &mut Population<G, A>, parent_indexes: &[usize], generation: i32) {
        let index_one = parent_indexes[0];
        let index_two = parent_indexes[1];

        let mut geno_one = population.get(index_one).genotype().clone();
        let mut geno_two = population.get(index_two).genotype().clone();

        let cross_count = self.cross_genotypes(&mut geno_one, &mut geno_two);

        if cross_count > 0 {
            population.set(index_one, Phenotype::from_genotype(geno_one, generation));
            population.set(index_two, Phenotype::from_genotype(geno_two, generation));
        }
    }

    #[inline]
    fn cross_genotypes(&self, geno_one: &mut Genotype<G, A>, geno_two: &mut Genotype<G, A>) -> i32 {
        let chromosome_index =
            rand::random::<usize>() % std::cmp::min(geno_one.len(), geno_two.len());

        let mut chrom_one = geno_one.get_chromosome_mut(chromosome_index);
        let mut chrom_two = geno_two.get_chromosome_mut(chromosome_index);

        self.cross_chromosomes(&mut chrom_one, &mut chrom_two)
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut Chromosome<G, A>,
        chrom_two: &mut Chromosome<G, A>,
    ) -> i32 {
        let rate = self.cross_rate();
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if rand::random::<f32>() < rate {
                let gene_one = chrom_one.get_gene(i);
                let gene_two = chrom_two.get_gene(i);

                let new_gene_one = gene_one.from_allele(gene_two.allele());
                let new_gene_two = gene_two.from_allele(gene_one.allele());

                chrom_one.set_gene(i, new_gene_one);
                chrom_two.set_gene(i, new_gene_two);

                cross_count += 1;
            }
        }

        cross_count
    }
}
