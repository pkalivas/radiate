use std::sync::Arc;

use super::EngineStep;
use crate::{
    Chromosome, Distance, GeneticEngineParams, Metric, Objective, Phenotype, Population,
    PopulationView, Score, Species, metric_names, random_provider,
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
        for species in species.iter_mut() {
            let random_phenotype = random_provider::choose(species.population().as_ref());

            species.set_mascot(random_phenotype.clone());
            species.population_mut().clear();
        }
    }

    pub fn fitness_share(&self, species: &mut Vec<Species<C>>) {
        let mut total_species_score = Score::default();
        for i in 0..species.len() {
            let members = species[i].adjusted_scores();
            total_species_score = total_species_score + members.iter().sum::<Score>();
        }

        for species in species.iter_mut() {
            let scaled_scores = species.adjusted_scores();
            let adjusted_score = scaled_scores.iter().sum::<Score>() / total_species_score.clone();

            self.objective.sort(species);

            let best_score = species.population().get(0).score().unwrap();
            species.update_score(adjusted_score, best_score, &self.objective);
        }

        species.sort_by(|a, b| self.objective.cmp(a.score(), b.score()));
    }
}

impl<C, T> EngineStep<C, T> for SpeciateStep<C>
where
    C: Chromosome,
    T: Clone + Send,
{
    fn execute(
        &self,
        generation: usize,
        population: &mut Population<C>,
        species: &mut Vec<Species<C>>,
    ) -> Vec<Metric> {
        let mut distances = Vec::new();

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
                let new_species = Species::new(Phenotype::clone(phenotype), generation);

                species.push(new_species);
            }
        }

        species.retain(|s| s.len() > 0);

        self.fitness_share(species);

        let species_count = species.len();

        vec![
            Metric::new_value(metric_names::SPECIATION).with_value(species_count as f32),
            Metric::new_distribution(metric_names::DISTANCE).with_distribution(&distances),
        ]
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
