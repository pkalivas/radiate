use std::sync::Arc;

use super::EngineStep;
use crate::domain::timer::Timer;
use crate::{
    Chromosome, Distance, EngineContext, GeneticEngineParams, Objective, Phenotype, Score, Species,
    metric_names, random_provider,
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

    pub fn generate_mascots(&self, species: &mut Vec<Species<C>>) {
        let mut to_remove = Vec::new();
        for (idx, species) in species.iter_mut().enumerate() {
            if species.len() == 0 {
                to_remove.push(idx);
                continue;
            }

            let random_phenotype = random_provider::choose(species.population().as_ref());

            species.set_mascot(random_phenotype.clone());
            species.population_mut().clear();
        }

        for idx in to_remove.iter().rev() {
            println!("Removing empty species at index {}", idx);
            species.remove(*idx);
        }
    }

    pub fn fitness_share(&self, species: &mut Vec<Species<C>>, objective: &Objective) {
        let mut total_species_score = Score::default();
        for i in 0..species.len() {
            let members = self.scaled_member_scores(&species[i].population().individuals);
            total_species_score = total_species_score + members.iter().sum::<Score>();
        }

        for species in species.iter_mut() {
            let scaled_scores = self.scaled_member_scores(&species.population().individuals);
            let adjusted_score = scaled_scores.iter().sum::<Score>() / total_species_score.clone();

            species.sort_by(objective);
            let best_score = species.population().get(0).score().unwrap();
            species.update_score(adjusted_score, best_score, objective);
        }

        species.sort_by(|a, b| objective.cmp(a.score(), b.score()));

        let scores = species
            .iter()
            .map(|species| species.score().clone())
            .collect::<Vec<Score>>();

        let mut idx = species.len();
        for i in 0..scores.len() {
            let species = &mut species[i];
            species.set_score(scores[idx - 1].clone());
            idx -= 1;
        }
    }

    fn scaled_member_scores(&self, members: &[Phenotype<C>]) -> Vec<Score> {
        members
            .iter()
            .map(|member| member.score().unwrap().clone() / members.len() as f32)
            .collect::<Vec<Score>>()
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

        let population = &mut ctx.population;
        let species = &mut ctx.species;

        self.generate_mascots(species);

        for i in 0..population.len() {
            let mut found = false;
            for j in 0..species.len() {
                let species = &mut species[j];
                let dist = self
                    .distance
                    .distance(&population[i].genotype(), &species.mascot().genotype());
                distances.push(dist);

                if dist < self.distance.threshold() {
                    species.add_member(population.get(i));
                    found = true;
                    break;
                }
            }

            if !found {
                let phenotype = population.get_mut(i);
                let new_species = Species::new(Phenotype::clone(phenotype), ctx.index);

                species.push(new_species);
            }
        }

        species.retain(|s| s.len() > 0);

        self.fitness_share(&mut ctx.species, &self.objective);

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
