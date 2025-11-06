use crate::{
    Optimize,
    objectives::{Objective, pareto},
};
use std::{cmp::Ordering, hash::Hash, ops::Range, sync::Arc};

/// A `Front<T>` is a collection of `T`'s that are non-dominated with respect to each other.
/// This is useful for multi-objective optimization problems where the goal is to find
/// the best solutions that are not dominated by any other solution.
/// This results in what is called the Pareto front.
#[derive(Clone)]
pub struct Front<T>
where
    T: AsRef<[f32]>,
{
    values: Vec<Arc<T>>,
    ord: Arc<dyn Fn(&T, &T) -> Ordering + Send + Sync>,
    range: Range<usize>,
    objective: Objective,
}

impl<T> Front<T>
where
    T: AsRef<[f32]>,
{
    pub fn new<F>(range: Range<usize>, objective: Objective, comp: F) -> Self
    where
        F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
    {
        Front {
            values: Vec::new(),
            range,
            objective,
            ord: Arc::new(comp),
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn objective(&self) -> Objective {
        self.objective.clone()
    }

    pub fn values(&self) -> &[Arc<T>] {
        &self.values
    }

    pub fn add_all(&mut self, items: &[T]) -> usize
    where
        T: Eq + Hash + Clone + Send + Sync + 'static,
    {
        let mut updated = false;
        let mut to_remove = Vec::new();
        let mut added_count = 0;

        for i in 0..items.len() {
            let new_member = &items[i];
            let mut is_dominated = true;

            for existing_val in self.values.iter() {
                let equals = new_member == existing_val.as_ref();
                if (self.ord)(existing_val.as_ref(), new_member) == Ordering::Greater || equals {
                    // If an existing value dominates the new value, return false
                    is_dominated = false;
                    break;
                } else if (self.ord)(new_member, existing_val.as_ref()) == Ordering::Greater {
                    // If the new value dominates an existing value, continue checking
                    to_remove.push(Arc::clone(existing_val));
                    continue;
                }
            }

            if is_dominated {
                updated = true;
                self.values.push(Arc::new(new_member.clone()));
                added_count += 1;
                for rem in to_remove.drain(..) {
                    self.values.retain(|x| x.as_ref() != rem.as_ref());
                }
            }

            if updated && self.values.len() > self.range.end {
                self.filter();
            }

            to_remove.clear();
            updated = false;
        }

        added_count
    }

    pub fn filter(&mut self) {
        let values = self.values.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
        let crowding_distances = pareto::crowding_distance(&values);

        let mut enumerated = crowding_distances.iter().enumerate().collect::<Vec<_>>();

        enumerated.sort_unstable_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(Ordering::Equal));

        self.values = enumerated
            .iter()
            .take(self.range.end)
            .map(|(i, _)| Arc::clone(&self.values[*i]))
            .collect::<Vec<Arc<T>>>();
    }
}

impl<T> Default for Front<T>
where
    T: AsRef<[f32]>,
{
    fn default() -> Self {
        Front::new(0..0, Objective::Single(Optimize::Minimize), |_, _| {
            std::cmp::Ordering::Equal
        })
    }
}
