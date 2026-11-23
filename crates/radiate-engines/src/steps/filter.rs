use crate::EvolutionContext;
use crate::steps::{EcosystemView, EngineStep};
use radiate_core::{
    Chromosome, EcosystemFilter, Genotype, Phenotype, ReplacementStrategy, Valid, metric_names,
};
use radiate_error::Result;
use std::sync::{Arc, Mutex};

pub struct FilterStep<C: Chromosome> {
    pub(crate) replacer: Arc<dyn ReplacementStrategy<C>>,
    pub(crate) encoder: Arc<dyn Fn() -> Genotype<C> + Send + Sync>,
    pub(crate) filters: Vec<Arc<Mutex<dyn EcosystemFilter<C>>>>,
    pub(crate) max_age: usize,
    pub(crate) max_species_age: usize,
}

impl<C: Chromosome, T> EngineStep<C, T> for FilterStep<C> {
    type View<'a>
        = EcosystemView<'a, C>
    where
        C: 'a,
        T: 'a;

    fn view<'a>(ctx: &'a mut EvolutionContext<C, T>) -> Self::View<'a> {
        EcosystemView::new(ctx)
    }

    #[inline]
    fn execute(&mut self, view: EcosystemView<'_, C>) -> Result<()> {
        let EcosystemView {
            index: generation,
            ecosystem,
            metrics,
        } = view;

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

                if let Some(phenotype) = ecosystem.get_phenotype_mut(i) {
                    *phenotype = Phenotype::from((new_genotype, generation));
                }
            }
        }

        if let Some(species) = ecosystem.species_mut() {
            let before_species = species.len();
            species.retain(|species| species.age(generation) < self.max_species_age);
            let species_count = before_species - species.len();

            if species_count > 0 {
                metrics.upsert(metric_names::SPECIES_AGE_FAIL, species_count);
            }
        }

        if age_count > 0 {
            metrics.upsert(metric_names::REPLACE_AGE, age_count);
        }

        if invalid_count > 0 {
            metrics.upsert(metric_names::REPLACE_INVALID, invalid_count);
        }

        for filter in self.filters.iter_mut() {
            let mut filter = filter.lock().unwrap();
            filter.filter(
                generation,
                ecosystem,
                metrics,
                Arc::clone(&self.replacer),
                Arc::clone(&self.encoder),
            )?;
        }

        Ok(())
    }
}
