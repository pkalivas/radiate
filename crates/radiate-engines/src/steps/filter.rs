use crate::steps::EngineStep;
use radiate_core::{
    Chromosome, Ecosystem, Genotype, MetricSet, Phenotype, ReplacementStrategy, Valid, metric_names,
};
use radiate_error::Result;
use std::sync::Arc;

pub struct FilterStep<C: Chromosome> {
    pub(crate) replacer: Arc<dyn ReplacementStrategy<C>>,
    pub(crate) encoder: Arc<dyn Fn() -> Genotype<C> + Send + Sync>,
    pub(crate) max_age: usize,
    pub(crate) max_species_age: usize,
}

impl<C: Chromosome> EngineStep<C> for FilterStep<C> {
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        let mut age_count = 0;
        let mut invalid_count = 0;
        for i in 0..ecosystem.population.len() {
            let removed = ecosystem
                .get_phenotype(i)
                .map(|pheno| {
                    let mut removed = false;
                    if pheno.age(generation) > self.max_age {
                        removed = true;
                        age_count += 1;
                    } else if !pheno.genotype().is_valid() {
                        removed = true;
                        invalid_count += 1;
                    }
                    removed
                })
                .unwrap_or(false);

            if removed {
                let new_genotype = self
                    .replacer
                    .replace(ecosystem.population(), Arc::clone(&self.encoder));

                ecosystem
                    .get_phenotype_mut(i)
                    .map(|pheno| *pheno = Phenotype::from((new_genotype, generation)));
            }
        }

        if let Some(species) = ecosystem.species_mut() {
            let before_species = species.len();
            species.retain(|species| species.age(generation) < self.max_species_age);
            let species_count = before_species - species.len();

            if species_count > 0 {
                metrics.upsert((metric_names::SPECIES_AGE_FAIL, species_count));
            }
        }

        if age_count > 0 {
            metrics.upsert((metric_names::REPLACE_AGE, age_count));
        }

        if invalid_count > 0 {
            metrics.upsert((metric_names::REPLACE_INVALID, invalid_count));
        }

        Ok(())
    }
}
