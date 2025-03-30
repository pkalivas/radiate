use super::{AlterAction, AlterResult};
use crate::{Chromosome, Gene, Genotype, Population, random_provider};

pub trait Mutate<C: Chromosome> {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>().split("::").last().unwrap()
    }

    fn rate(&self) -> f32 {
        1.0
    }

    fn alterer(self) -> AlterAction<C>
    where
        Self: Sized + 'static,
    {
        AlterAction::Mutate(self.name(), self.rate(), Box::new(self))
    }

    #[inline]
    fn mutate(&self, population: &mut Population<C>, rate: f32) -> AlterResult {
        let mut result = AlterResult::default();

        for (idx, phenotype) in population.iter_mut().enumerate() {
            let mutate_result = self.mutate_genotype(&mut phenotype.genotype_mut(), rate);

            if mutate_result.count() > 0 {
                result.mark_changed(idx);
            }

            result.merge(mutate_result);
        }

        result
    }

    #[inline]
    fn mutate_genotype(&self, genotype: &mut Genotype<C>, rate: f32) -> AlterResult {
        let mut result = AlterResult::default();

        for chromosome in genotype.iter_mut() {
            let mutate_result = self.mutate_chromosome(chromosome, rate);
            result.merge(mutate_result);
        }

        result
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::random::<f32>() < rate {
                *gene = self.mutate_gene(gene);
                count += 1;
            }
        }

        count.into()
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        gene.new_instance()
    }
}
