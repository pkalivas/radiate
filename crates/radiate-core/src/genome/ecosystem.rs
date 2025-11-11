use super::{Chromosome, Genotype, Phenotype, Population, Species};
use crate::{Objective, Score, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ecosystem<C: Chromosome> {
    pub population: Population<C>,
    pub species: Option<Vec<Species<C>>>,
}

impl<C: Chromosome> Ecosystem<C> {
    pub fn new(population: Population<C>) -> Self {
        Ecosystem {
            population,
            species: None,
        }
    }

    pub fn clone_ref(other: &Ecosystem<C>) -> Self
    where
        C: Clone,
    {
        Ecosystem {
            population: Population::clone_ref(&other.population),
            species: other.species.as_ref().map(|specs| {
                specs
                    .iter()
                    .map(|species| Species::clone_ref(species))
                    .collect()
            }),
        }
    }

    pub fn population(&self) -> &Population<C> {
        &self.population
    }

    pub fn population_mut(&mut self) -> &mut Population<C> {
        &mut self.population
    }

    pub fn species(&self) -> Option<&Vec<Species<C>>> {
        self.species.as_ref()
    }

    pub fn species_mut(&mut self) -> Option<&mut Vec<Species<C>>> {
        self.species.as_mut()
    }

    pub fn get_phenotype(&self, index: usize) -> Option<&Phenotype<C>> {
        self.population.get(index)
    }

    pub fn get_phenotype_mut(&mut self, index: usize) -> Option<&mut Phenotype<C>> {
        self.population.get_mut(index)
    }

    pub fn get_genotype(&self, index: usize) -> Option<&Genotype<C>> {
        self.population.get(index).map(|p| p.genotype())
    }

    pub fn get_genotype_mut(&mut self, index: usize) -> Option<&mut Genotype<C>> {
        self.population.get_mut(index).map(|p| p.genotype_mut())
    }

    pub fn get_species(&self, index: usize) -> Option<&Species<C>> {
        self.species.as_ref().and_then(|s| s.get(index))
    }

    pub fn get_species_mut(&mut self, index: usize) -> Option<&mut Species<C>> {
        self.species.as_mut().and_then(|s| s.get_mut(index))
    }

    pub fn species_mascots(&self) -> Vec<&Phenotype<C>> {
        self.species
            .as_ref()
            .map(|s| s.iter().map(|spec| spec.mascot()).collect())
            .unwrap_or_default()
    }

    pub fn push_species(&mut self, species: Species<C>) {
        if let Some(species_list) = &mut self.species {
            species_list.push(species);
        } else {
            self.species = Some(vec![species]);
        }
    }

    pub fn add_species_member(&mut self, species_idx: usize, member_idx: usize)
    where
        C: Clone,
    {
        if let Some(species) = &mut self.species {
            if let Some(spec) = species.get_mut(species_idx) {
                if let Some(member) = self.population.ref_clone_member(member_idx) {
                    spec.population.push(member);
                }
            }
        }
    }

    pub fn generate_mascots(&mut self)
    where
        C: Clone,
    {
        if let Some(species) = &mut self.species {
            for spec in species {
                let idx = random_provider::range(0..spec.population.len());
                spec.mascot = spec.population.get(idx).unwrap().clone();
                spec.population.clear();
            }
        }
    }

    pub fn fitness_share(&mut self, objective: &Objective)
    where
        C: PartialEq,
    {
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

    fn adjust_scores(species: &Species<C>) -> Vec<Score> {
        species
            .population
            .get_scores()
            .map(|score| (*score).clone() / species.len() as f32)
            .collect()
    }
}

impl<C: Chromosome + Clone> Clone for Ecosystem<C> {
    fn clone(&self) -> Self {
        Ecosystem {
            population: self.population.clone(),
            species: self.species.clone(),
        }
    }
}
