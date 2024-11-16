use super::{chromosome::Chromosome, genes::gene::Gene};

pub struct Genotype<G, A>
where
    G: Gene<G, A>,
{
    pub chromosomes: Vec<Chromosome<G, A>>,
}

impl<G, A> Genotype<G, A>
where
    G: Gene<G, A>,
{
    pub fn from_chromosomes(chromosomes: Vec<Chromosome<G, A>>) -> Self {
        Genotype { chromosomes }
    }

    pub fn get_chromosome_mut(&mut self, index: usize) -> &mut Chromosome<G, A> {
        &mut self.chromosomes[index]
    }

    pub fn get_chromosome(&self, index: usize) -> &Chromosome<G, A> {
        &self.chromosomes[index]
    }

    pub fn set_chromosome(&mut self, index: usize, chromosome: Chromosome<G, A>) {
        self.chromosomes[index] = chromosome;
    }

    pub fn len(&self) -> usize {
        self.chromosomes.len()
    }

    pub fn is_valid(&self) -> bool {
        self.chromosomes
            .iter()
            .all(|chromosome| chromosome.is_valid())
    }

    pub fn iter(&self) -> std::slice::Iter<Chromosome<G, A>> {
        self.chromosomes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Chromosome<G, A>> {
        self.chromosomes.iter_mut()
    }
}

impl<G, A> Clone for Genotype<G, A>
where
    G: Gene<G, A>,
{
    fn clone(&self) -> Self {
        Genotype {
            chromosomes: self.chromosomes.clone(),
        }
    }
}

impl<G, A> PartialEq for Genotype<G, A>
where
    G: Gene<G, A>,
{
    fn eq(&self, other: &Self) -> bool {
        self.chromosomes == other.chromosomes
    }
}

impl<G, A> std::fmt::Debug for Genotype<G, A>
where
    G: Gene<G, A> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for chromosome in &self.chromosomes {
            write!(f, "{:?},\n ", chromosome)?;
        }
        write!(f, "]")
    }
}
