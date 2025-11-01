use super::TreeChromosome;
use radiate_core::{AlterResult, Mutate, random_provider};

#[derive(Clone, Debug)]
pub struct HoistMutator {
    rate: f32,
}

impl HoistMutator {
    pub fn new(rate: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("rate must be between 0.0 and 1.0");
        }

        HoistMutator { rate }
    }
}

impl<T> Mutate<TreeChromosome<T>> for HoistMutator
where
    T: Clone + PartialEq,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    fn mutate_chromosome(&self, chromosome: &mut TreeChromosome<T>, _: f32) -> AlterResult {
        let root = chromosome.root_mut();
        let root_size = root.size();
        let rand_index = random_provider::range(0..root_size);

        if rand_index < 1 {
            return AlterResult::empty();
        }

        if let Some(rand_node) = root.get_mut(rand_index) {
            if rand_node.children().is_none() {
                return AlterResult::empty();
            }

            let child_idx = random_provider::range(0..rand_node.children().map_or(0, |c| c.len()));
            let mut child = rand_node.detach(child_idx);

            return if let Some(child) = child.as_mut() {
                std::mem::swap(rand_node, child);
                AlterResult::from(1)
            } else {
                AlterResult::empty()
            };
        }

        AlterResult::empty()
    }
}
