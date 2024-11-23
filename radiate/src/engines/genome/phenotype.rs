use crate::engines::score::Score;

use super::{genes::gene::Gene, genotype::Genotype, Valid};

/// A ```Phenotype``` is a representation of an individual in the population. It contains:
/// * ```Genotype``` - the genetic representation of the individual
/// * ```Score``` - the score (fitness) of the individual as calculated by the fitness function
/// * ```Generation``` - the generation in which the individual was created
///
/// The ```Phenotype``` is a wrapper around the ```Genotype``` that adds additional information about the individual.
/// In traditional (biological) genetics, a phenotype is "the set of observable characteristics of an individual resulting 
/// from the interaction of its genotype with the environment". As such, the ```Phenotype``` is the 'observable' part of the
/// individual (```Genotype```) that is being evolved by the genetic algorithm, hense the ```Score``` and ```Generation``` fields.
/// This allows the ```Phenotype``` to be sorted and compared based on the fitness (```Score```) of the individual (```Genotype```)
/// 
/// # Type Parameters
/// - `G`: The type of gene used in the genetic algorithm, which must implement the `Gene` trait.
/// - `A`: The type of the allele associated with the gene - the gene's "expression".
pub struct Phenotype<G, A>
where
    G: Gene<G, A>,
{
    pub genotype: Genotype<G, A>,
    pub score: Option<Score>,
    pub generation: i32,
}

impl<G, A> Phenotype<G, A>
where
    G: Gene<G, A>,
{
    /// Create a new instance of the ```Phenotype``` with the given genotype and generation. The score is set to None.
    pub fn from_genotype(genotype: Genotype<G, A>, generation: i32) -> Self {
        Phenotype {
            genotype,
            score: None,
            generation,
        }
    }

    pub fn genotype(&self) -> &Genotype<G, A> {
        &self.genotype
    }

    pub fn genotype_mut(&mut self) -> &mut Genotype<G, A> {
        &mut self.genotype
    }

    pub fn score(&self) -> &Option<Score> {
        &self.score
    }

    pub fn set_score(&mut self, score: Option<Score>) {
        self.score = score;
    }

    /// Get the age of the individual in generations. The age is calculated as the 
    /// difference between the given generation and the generation in which the individual was created.
    pub fn age(&self, generation: i32) -> i32 {
        generation - self.generation
    }
}

/// Implement the ```Valid``` trait for the ```Phenotype```. This allows the ```Phenotype``` to be checked for validity.
/// A ```Phenotype``` is valid if the ```Genotype``` is valid. The ```GeneticEngine``` checks the validity of the ```Phenotype```
/// and will remove any invalid individuals from the population, replacing them with new individuals at the given generation.
impl<G, A> Valid for Phenotype<G, A>
where
    G: Gene<G, A>,
{
    fn is_valid(&self) -> bool {
        self.genotype.is_valid()
    }
}

impl<G, A> Clone for Phenotype<G, A>
where
    G: Gene<G, A>,
{
    fn clone(&self) -> Self {
        Phenotype {
            genotype: self.genotype.clone(),
            score: match &self.score {
                Some(score) => Some(score.clone()),
                None => None,
            },
            generation: self.generation,
        }
    }
}

impl<G, A> PartialEq for Phenotype<G, A>
where
    G: Gene<G, A>,
{
    fn eq(&self, other: &Self) -> bool {
        self.genotype == other.genotype
            && self.score == other.score
            && self.generation == other.generation
    }
}

/// Implement the ```PartialOrd``` trait for the ```Phenotype```. This allows the ```Phenotype``` to be compared
/// with other ```Phenotype``` instances. The comparison is based on the ```Score``` (fitness) of the ```Phenotype```.
impl<G, A> PartialOrd for Phenotype<G, A>
where
    G: Gene<G, A>,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<G, A> std::fmt::Debug for Phenotype<G, A>
where
    G: Gene<G, A> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}, generation: {:?}, score: {:?}",
            self.genotype, self.generation, self.score
        )
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::engines::genome::chromosome::Chromosome;
    use crate::engines::genome::genes::float_gene::FloatGene;

    #[test]
    fn test_from_genotype() {
        let genotype =
            Genotype::from_chromosomes(vec![Chromosome::from_genes(vec![FloatGene::new(
                0_f32, 1_f32,
            )])]);

        let phenotype = Phenotype::from_genotype(genotype.clone(), 0);
        assert_eq!(phenotype.genotype, genotype);
        assert_eq!(phenotype.generation, 0);
        assert_eq!(phenotype.score, None);
    }

    #[test]
    fn test_age() {
        let genotype =
            Genotype::from_chromosomes(vec![Chromosome::from_genes(vec![FloatGene::new(
                0_f32, 1_f32,
            )])]);

        let phenotype = Phenotype::from_genotype(genotype.clone(), 0);
        assert_eq!(phenotype.age(10), 10);
    }
}
