use super::{Chromosome, Gene, Valid};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};

/// The [PermutationGene] is a gene that represents a permutation of a set of alleles. The gene has an index
/// that represents the position of the allele in the alleles vector. The alleles vector is a set of unique
/// values. The gene is valid if the index is less than the length of the alleles vector. This gene is useful
/// for representing permutations of values, such as the order of cities in a TSP problem.
///
/// # Type Parameters
/// - `A`: The type of the alleles.
#[derive(Debug, Clone, PartialEq)]
pub struct PermutationGene<A: PartialEq + Clone> {
    index: usize,
    alleles: Arc<[A]>,
}

impl<A: PartialEq + Clone> PermutationGene<A> {
    pub fn new(index: usize, alleles: Arc<[A]>) -> Self {
        PermutationGene { index, alleles }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn with_index(&self, index: usize) -> Self {
        PermutationGene {
            index,
            alleles: Arc::clone(&self.alleles),
        }
    }
}

impl<A: PartialEq + Clone> Gene for PermutationGene<A> {
    type Allele = A;

    fn allele(&self) -> &Self::Allele {
        &self.alleles[self.index]
    }

    fn allele_mut(&mut self) -> &mut Self::Allele {
        panic!(
            "Cannot mutate allele of PermutationGene directly. Create a new gene with `with_allele` or `with_index`."
        );
    }

    fn new_instance(&self) -> Self {
        PermutationGene {
            index: self.index,
            alleles: Arc::clone(&self.alleles),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        // Find the index of the allele in the alleles vector - this implies that `self.alleles`
        // is a set of unique values.
        let index = self.alleles.iter().position(|x| x == allele).unwrap();
        PermutationGene {
            index,
            alleles: Arc::clone(&self.alleles),
        }
    }
}

impl<A: PartialEq + Clone> Valid for PermutationGene<A> {
    fn is_valid(&self) -> bool {
        self.index < self.alleles.len()
    }
}

#[cfg(feature = "serde")]
impl<A: PartialEq + Clone> Serialize for PermutationGene<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.index as u64)
    }
}

#[cfg(feature = "serde")]
impl<'de, A: PartialEq + Clone> Deserialize<'de> for PermutationGene<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let index =
            usize::try_from(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)?;
        Ok(PermutationGene {
            index,
            alleles: vec![].into_boxed_slice().into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PermutationChromosome<A: PartialEq + Clone> {
    pub genes: Vec<PermutationGene<A>>,
    alleles: Arc<[A]>,
}

impl<A: PartialEq + Clone> PermutationChromosome<A> {
    pub fn new(genes: Vec<PermutationGene<A>>, alleles: Arc<[A]>) -> Self {
        PermutationChromosome { genes, alleles }
    }

    pub fn alleles(&self) -> &Arc<[A]> {
        &self.alleles
    }
}

impl<A: PartialEq + Clone> Chromosome for PermutationChromosome<A> {
    type Gene = PermutationGene<A>;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

impl<A: PartialEq + Clone> Valid for PermutationChromosome<A> {
    fn is_valid(&self) -> bool {
        // Check if the genes are a valid permutation of the alleles
        let mut bit_set = vec![false; self.alleles.len()];
        self.genes.iter().all(|gene| {
            let index = gene.index;
            if bit_set[index] {
                return false;
            }
            bit_set[index] = true;
            true
        })
    }
}

impl<A: PartialEq + Clone> From<Vec<PermutationGene<A>>> for PermutationChromosome<A> {
    fn from(genes: Vec<PermutationGene<A>>) -> Self {
        let alleles = genes
            .first()
            .map(|g| Arc::clone(&g.alleles))
            .unwrap_or_default();
        PermutationChromosome { genes, alleles }
    }
}

impl<A: PartialEq + Clone> IntoIterator for PermutationChromosome<A> {
    type Item = PermutationGene<A>;
    type IntoIter = std::vec::IntoIter<PermutationGene<A>>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

#[cfg(feature = "serde")]
impl<A: PartialEq + Clone + Serialize> Serialize for PermutationChromosome<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PermutationChromosome", 2)?;
        state.serialize_field("alleles", &*self.alleles)?;
        let gene_indices: Vec<usize> = self.genes.iter().map(|g| g.index).collect();
        state.serialize_field("indices", &gene_indices)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, A: PartialEq + Clone + Deserialize<'de>> Deserialize<'de> for PermutationChromosome<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PermutationChromosomeData<A> {
            alleles: Vec<A>,
            indices: Vec<usize>,
        }

        let data = PermutationChromosomeData::<A>::deserialize(deserializer)?;
        let alleles = data.alleles.into_boxed_slice().into();
        let genes = data
            .indices
            .into_iter()
            .map(|index| PermutationGene {
                index,
                alleles: Arc::clone(&alleles),
            })
            .collect();

        Ok(PermutationChromosome { genes, alleles })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_permutation_gene() {
        let alleles = Arc::new([1, 2, 3, 4]);
        let gene = PermutationGene::new(0, alleles);

        assert_eq!(gene.allele(), &1);
        assert!(gene.is_valid());
    }

    #[test]
    fn test_permutation_chromosome() {
        let alleles: Arc<[i32]> = Arc::new([1, 2, 3, 4]);
        let genes = vec![
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(3, Arc::clone(&alleles)),
        ];
        let chromosome = PermutationChromosome::new(genes.clone(), Arc::clone(&alleles));

        assert_eq!(chromosome.genes.len(), 4);
        assert!(chromosome.is_valid());
        for (i, gene) in chromosome.genes.iter().enumerate() {
            assert_eq!(gene.index, i);
            assert_eq!(gene.allele(), &alleles[i]);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_deserialize_gene() {
        let alleles: Arc<[char]> = Arc::new(['A', 'B', 'C']);
        let gene = PermutationGene::new(1, Arc::clone(&alleles));

        // Serialize
        let encoded = serde_json::to_string(&gene).expect("serialize gene failed");

        // Deserialize
        let mut deserialized: PermutationGene<char> =
            serde_json::from_str(&encoded).expect("deserialize gene failed");

        // Manually reattach alleles (since PermutationGene deserialization uses dummy alleles)
        deserialized.alleles = Arc::clone(&alleles);

        assert_eq!(deserialized.index, gene.index);
        assert_eq!(deserialized.allele(), gene.allele());
        assert!(deserialized.is_valid());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize_deserialize_chromosome() {
        let alleles: Arc<[char]> = Arc::new(['X', 'Y', 'Z']);
        let genes = vec![
            PermutationGene::new(2, Arc::clone(&alleles)),
            PermutationGene::new(0, Arc::clone(&alleles)),
            PermutationGene::new(1, Arc::clone(&alleles)),
        ];
        let chromosome = PermutationChromosome::new(genes, Arc::clone(&alleles));

        let encoded = serde_json::to_string(&chromosome).expect("serialize chromosome failed");
        let deserialized: PermutationChromosome<char> =
            serde_json::from_str(&encoded).expect("deserialize chromosome failed");

        assert_eq!(deserialized.alleles.as_ref(), alleles.as_ref());
        assert_eq!(deserialized.genes.len(), chromosome.genes.len());
        assert!(deserialized.is_valid());

        for (gene, expected_gene) in deserialized.genes.iter().zip(chromosome.genes.iter()) {
            assert_eq!(gene.index, expected_gene.index);
            assert_eq!(gene.allele(), expected_gene.allele());
        }
    }
}
