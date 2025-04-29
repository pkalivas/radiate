use super::{Chromosome, Phenotype, Population, Species};
use crate::{Objective, Score, random_provider};

pub struct Ecosystem<C: Chromosome> {
    pub population: Population<C>,
    pub species: Option<Vec<Species<C>>>,
    pub is_sorted: bool,
}

impl<C: Chromosome> Ecosystem<C> {
    pub fn new(population: Population<C>) -> Self {
        Ecosystem {
            population,
            species: None,
            is_sorted: false,
        }
    }

    pub fn generate_mascots(&mut self) {
        if let Some(species) = &mut self.species {
            for spec in species {
                let mascot = random_provider::choose(&spec.population.as_ref());
                spec.mascot = mascot.get().clone();
                spec.population.clear();
            }
        }
    }

    pub fn fitness_share(&mut self, objective: &Objective) {
        if let Some(species) = &mut self.species {
            let mut scores = Vec::with_capacity(species.len());
            for spec in species.iter() {
                scores.push(Self::adjust_scores(spec).iter().sum::<Score>());
            }

            let total_score = scores.iter().sum::<Score>();
            for (i, spec) in species.iter_mut().enumerate() {
                let spec_score = scores[i].clone();
                let adjusted_score = spec_score / total_score.clone();
                spec.update_score(adjusted_score, objective);
            }

            objective.sort(species);
        }
    }

    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Phenotype<C>, &Phenotype<C>) -> std::cmp::Ordering,
    {
        if self.is_sorted {
            return;
        }

        self.population.sort_by(f);
        self.is_sorted = true;
    }

    fn adjust_scores(species: &Species<C>) -> Vec<Score> {
        species
            .population
            .get_scores()
            .iter()
            .map(|score| (*score).clone() / species.len() as f32)
            .collect()
    }
}
