use std::sync::Arc;

use super::EngineStep;
use crate::domain::timer::Timer;
use crate::{
    Chromosome, Distance, EngineContext, GeneticEngineParams, Objective, Species, metric_names,
    species,
};

pub struct SpeciateStep<C: Chromosome> {
    objective: Objective,
    distance: Arc<dyn Distance<C>>,
}

impl<C: Chromosome> SpeciateStep<C> {
    pub fn new(objective: Objective, distance: Arc<dyn Distance<C>>) -> Self {
        SpeciateStep {
            objective,
            distance,
        }
    }
}

impl<C, T> EngineStep<C, T> for SpeciateStep<C>
where
    C: Chromosome,
    T: Clone + Send,
{
    fn execute(&self, ctx: &mut EngineContext<C, T>) {
        let timer = Timer::new();
        let mut distances = Vec::new();

        species::generate_mascots(&mut ctx.population, &mut ctx.species);

        for i in 0..ctx.population.len() {
            let mut found = false;
            for j in 0..ctx.species.len() {
                let species = ctx.get_species(j);
                let dist = self
                    .distance
                    .distance(&ctx.population[i].genotype(), species.mascot());
                distances.push(dist);

                if dist < self.distance.threshold() {
                    ctx.set_species_id(i, species.id());
                    found = true;
                    break;
                }
            }

            if !found {
                let phenotype = ctx.population.get_mut(i);
                let genotype = phenotype.genotype().clone();
                let score = phenotype.score().unwrap().clone();
                let new_species = Species::new(genotype, score, ctx.index);

                phenotype.set_species_id(Some(new_species.id()));
                ctx.add_species(new_species);
            }
        }

        species::fitness_share(&mut ctx.population, &mut ctx.species, &self.objective);

        let species_count = ctx.species().len();
        ctx.record_operation(metric_names::SPECIATION, species_count as f32, timer);
        ctx.record_distribution(metric_names::DISTANCE, &distances);
    }

    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        if let Some(distance) = params.distance() {
            return Some(Box::new(SpeciateStep {
                objective: params.objective().clone(),
                distance: Arc::clone(&distance),
            }));
        }
        None
    }
}
