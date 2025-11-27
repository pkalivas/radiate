use super::{Chromosome, Genotype, Phenotype, Population, Species};
use crate::{Objective, Score, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
/// let codec = FloatCodec::vector(10, 0.0..1.0);
/// let population = (0..100)
///    .map(|_| Phenotype::from((codec.encode(), 0)))
///    .collect::<Population<FloatChromosome>>();
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

    /// Reference clone an existing ecosystem. This creates a new ecosystem
    /// with reference counted clones of the phenotypes and species. This just clones
    /// pointers to the underlying [Population] and [Species], so any modifications to
    /// the phenotypes will be reflected in both ecosystems. Radiate uses this internally
    /// for efficiency when iterating over generations - this is not intended for general use.
    ///
    /// **Use with caution**
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

    /// Add a member to a species given the species index and member index in the population.
    /// The member is reference cloned from the population and added to the species' population.
    /// Just like with the [Ecosystem]'s `clone_ref` method, this creates a shared reference so
    /// any modifications to the phenotype within the [Species] will be reflected in the main [Population].
    ///
    /// **Use with caution**
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

    /// Generate mascots for each species by randomly selecting a member from the species' population.
    /// The selected member is deep cloned and set as the species' mascot. After selecting the mascot,
    /// the species' population is cleared for the next generation.
    pub fn generate_mascots(&mut self)
    where
        C: Clone,
    {
        if let Some(species) = &mut self.species {
            for spec in species {
                let idx = random_provider::range(0..spec.population.len());
                if let Some(phenotype) = spec.population.get(idx) {
                    spec.mascot = phenotype.clone();
                    spec.population.clear();
                }
            }
        }
    }

    /// Apply fitness sharing to the species in the ecosystem. This method
    /// adjusts the scores of each species based on the number of members
    /// in the species. The adjusted score is calculated by dividing the sum of the species'
    /// member scores by the number of members in the species.
    /// The adjusted scores are then used during selection to promote diversity.
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
