use super::{Chromosome, Genotype, Phenotype, Population, Species};
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

    /// Get the number of shared phenotypes in the population.
    /// A shared phenotype is one that is reference cloned and held
    /// by another structure, such as a [Species]. The only time the shared
    /// count should ever be > 0 is when the ecosystem has [Species] that
    /// hold references to phenotypes in the main population.
    pub fn shared_count(&self) -> usize {
        self.population.shared_count()
    }

    /// Like [Ecosystem::shared_count], but returns true if there are
    /// any shared phenotypes in the population. This should only be true
    /// when the ecosystem has [Species] that hold references to phenotypes
    /// in the main population.
    pub fn is_shared(&self) -> bool {
        self.shared_count() > 0
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

    pub fn remove_dead_species(&mut self) -> usize {
        if let Some(species) = &mut self.species {
            let initial_len = species.len();
            species.retain(|spec| spec.len() > 0);
            initial_len - species.len()
        } else {
            0
        }
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
