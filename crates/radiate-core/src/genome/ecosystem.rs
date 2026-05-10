use super::{Chromosome, Genotype, Phenotype, Population, Species};
use crate::{Objective, Score, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// An ecosystem containing a population and optional species.
/// This structure is the main container for solutions generated throughout the evolutionary process.
/// The optional species field allows for organizing the population into distinct groups,
/// each represented by a mascot phenotype.
///
/// When using [Species] within the ecosystem, it is important to manage the population
/// members appropriately. Species hold shared references to phenotypes in the main population,
/// so any modifications to the population should ensure that these references remain valid.
///
/// # Example
/// ```rust
/// use radiate_core::*;
///
/// // Create a simple ecosystem
/// let codec = FloatCodec::vector(10, 0.0_f64..1.0_f64);
/// let population = (0..100)
///    .map(|_| Phenotype::from((codec.encode(), 0)))
///    .collect::<Population<FloatChromosome<f64>>>();
///
/// let ecosystem = Ecosystem::new(population);
/// ```
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

    pub fn len(&self) -> usize {
        self.population.len()
    }

    pub fn is_empty(&self) -> bool {
        self.population.is_empty()
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

    pub fn species_population_mut(&mut self) -> (Option<&mut Vec<Species<C>>>, &mut Population<C>) {
        (self.species.as_mut(), &mut self.population)
    }

    pub fn species_mascots(&self) -> Vec<&Phenotype<C>> {
        self.species
            .as_ref()
            .map(|s| s.iter().map(|spec| spec.mascot()).collect())
            .unwrap_or_default()
    }

    pub fn push_species(&mut self, species: Species<C>) -> usize {
        if let Some(species_list) = &mut self.species {
            species_list.push(species);
            species_list.len() - 1
        } else {
            self.species = Some(vec![species]);
            0
        }
    }

    /// Add a member to a species given the species index and member index in the population.
    /// The member is reference cloned from the population and added to the species' population.
    /// Just like with the [Ecosystem]'s `clone_ref` method, this creates a shared reference so
    /// any modifications to the phenotype within the [Species] will be reflected in the main [Population].
    pub fn add_species_member(&mut self, species_idx: usize, member_idx: usize) {
        if let Some(species) = &mut self.species
            && let Some(spec) = species.get_mut(species_idx)
            && let Some(member) = self.population.get_mut(member_idx)
        {
            member.set_species(spec.id());
            spec.add_member(member.id());
        }
    }

    pub fn remove_dead_species(&mut self) -> usize {
        if let Some(species) = &mut self.species {
            let initial_len = species.len();
            let unique_species_ids = self
                .population
                .iter()
                .map(|p| p.species())
                .collect::<HashSet<_>>();
            species.retain(|spec| unique_species_ids.contains(&spec.id()));
            initial_len - species.len()
        } else {
            0
        }
    }

    pub fn generate_mascots(&mut self)
    where
        C: Clone,
    {
        // Update mascots for each species by selecting a random member from the species population
        // to be the new mascot for the next generation. This follows the NEAT algorithm approach.
        if let Some(species) = &mut self.species {
            for spec in species.iter_mut() {
                let species_members = self
                    .population
                    .iter_species(spec.id())
                    .collect::<Vec<&Phenotype<C>>>();

                if species_members.is_empty() {
                    continue;
                }

                let idx = random_provider::range(0..species_members.len());
                if let Some(phenotype) = species_members.get(idx) {
                    spec.set_new_mascot((*phenotype).clone());
                }
            }
        }
    }

    #[inline]
    pub fn fitness_share(&mut self, objective: &Objective)
    where
        C: PartialEq,
    {
        if let Some(species) = &mut self.species {
            let mut adjusted_scores = Vec::with_capacity(species.len());
            let mut raw_scores = Vec::with_capacity(species.len());
            for spec in species.iter() {
                let species_member_scores = self
                    .population
                    .iter_species(spec.id())
                    .filter_map(|pheno| pheno.score())
                    .collect::<Vec<&Score>>();

                let adjusted = Self::adjust_scores(&species_member_scores).sum::<Score>();

                raw_scores.push(species_member_scores[0]);
                adjusted_scores.push(adjusted);
            }

            let total_score = adjusted_scores.iter().sum::<Score>();
            for (i, spec) in species.iter_mut().enumerate() {
                let raw_score = raw_scores[i].clone();
                let spec_score = adjusted_scores[i].clone();
                let adjusted_score = spec_score / total_score.clone();
                spec.update_score(raw_score, adjusted_score, objective);
            }

            objective.sort(species);
        }
    }

    #[inline]
    fn adjust_scores(scores: &[&Score]) -> impl Iterator<Item = Score> {
        scores
            .iter()
            .map(|score| (**score).clone() / scores.len() as f32)
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

impl<C: Chromosome> From<Vec<Phenotype<C>>> for Ecosystem<C> {
    fn from(phenotypes: Vec<Phenotype<C>>) -> Self {
        Ecosystem {
            population: Population::from(phenotypes),
            species: None,
        }
    }
}

impl<C: Chromosome> From<Population<C>> for Ecosystem<C> {
    fn from(population: Population<C>) -> Self {
        Ecosystem {
            population,
            species: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_create_ecosystem() {
        let codec = FloatCodec::vector(5, 0.0..1.0);
        let phenotypes = (0..10)
            .map(|_| Phenotype::from((codec.encode(), 0)))
            .collect::<Vec<_>>();

        let ecosystem = Ecosystem::from(phenotypes);
        assert_eq!(ecosystem.population.len(), 10);
        assert!(ecosystem.species.is_none());
    }
}
