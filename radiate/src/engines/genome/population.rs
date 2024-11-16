use super::{genes::gene::Gene, phenotype::Phenotype};

pub struct Population<G, A>
where
    G: Gene<G, A>,
{
    pub individuals: Vec<Phenotype<G, A>>,
    pub is_sorted: bool,
}

impl<G, A> Population<G, A>
where
    G: Gene<G, A>,
{
    pub fn get(&self, index: usize) -> &Phenotype<G, A> {
        self.individuals.get(index).expect("Index out of bounds")
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Phenotype<G, A> {
        self.is_sorted = false;
        self.individuals
            .get_mut(index)
            .expect("Index out of bounds")
    }

    pub fn set(&mut self, index: usize, individual: Phenotype<G, A>) {
        self.individuals[index] = individual;
        self.is_sorted = false;
    }

    pub fn iter(&self) -> std::slice::Iter<Phenotype<G, A>> {
        self.individuals.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Phenotype<G, A>> {
        self.is_sorted = false;
        self.individuals.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Phenotype<G, A>, &Phenotype<G, A>) -> std::cmp::Ordering,
    {
        if self.is_sorted {
            return;
        }

        self.individuals.sort_by(f);
        self.is_sorted = true;
    }

    pub fn from_vec(individuals: Vec<Phenotype<G, A>>) -> Self {
        Population {
            individuals,
            is_sorted: false,
        }
    }

    pub fn from_fn<F>(size: usize, f: F) -> Self
    where
        F: Fn() -> Phenotype<G, A>,
    {
        let mut individuals = Vec::with_capacity(size);
        for _ in 0..size {
            individuals.push(f());
        }

        Population {
            individuals,
            is_sorted: false,
        }
    }
}

impl<G, A> IntoIterator for Population<G, A>
where
    G: Gene<G, A>,
{
    type Item = Phenotype<G, A>;
    type IntoIter = std::vec::IntoIter<Phenotype<G, A>>;

    fn into_iter(self) -> Self::IntoIter {
        self.individuals.into_iter()
    }
}

impl<G, A> FromIterator<Phenotype<G, A>> for Population<G, A>
where
    G: Gene<G, A>,
{
    fn from_iter<I: IntoIterator<Item = Phenotype<G, A>>>(iter: I) -> Self {
        let individuals = iter.into_iter().collect();
        Population {
            individuals,
            is_sorted: false,
        }
    }
}

impl<G, A> Clone for Population<G, A>
where
    G: Gene<G, A>,
{
    fn clone(&self) -> Self {
        Population {
            individuals: self.individuals.clone(),
            is_sorted: self.is_sorted,
        }
    }
}

impl<G, A> std::fmt::Debug for Population<G, A>
where
    G: Gene<G, A> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for individual in &self.individuals {
            write!(f, "{:?},\n ", individual)?;
        }
        write!(f, "]")
    }
}
