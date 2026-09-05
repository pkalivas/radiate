use super::TreeChromosome;
use radiate_core::{AlterContext, AlterCount, Expr, Mutate, RateSet, random_provider};

#[derive(Clone, Debug)]
pub struct HoistMutator {
    rate: Expr,
}

impl HoistMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        HoistMutator { rate: rate.into() }
    }
}

impl<T> Mutate<TreeChromosome<T>> for HoistMutator
where
    T: Clone + PartialEq,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    fn mutate_chromosome(
        &mut self,
        chromosome: &mut TreeChromosome<T>,
        _: &mut AlterContext,
    ) -> AlterCount {
        let root = chromosome.root_mut();
        let root_size = root.size();
        let rand_index = random_provider::range(0..root_size);

        if rand_index < 1 {
            return AlterCount::empty();
        }

        if let Some(rand_node) = root.get_mut(rand_index) {
            if rand_node.children().is_none() {
                return AlterCount::empty();
            }

            let child_idx = random_provider::range(0..rand_node.children().map_or(0, |c| c.len()));
            let mut child = rand_node.detach(child_idx);

            return if let Some(child) = child.as_mut() {
                std::mem::swap(rand_node, child);
                AlterCount::from(1)
            } else {
                AlterCount::empty()
            };
        }

        AlterCount::empty()
    }
}
