use crate::{
    Optimize,
    objectives::{Objective, Scored, pareto},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, hash::Hash, ops::Range, sync::Arc};

/// A `Front<T>` is a collection of `T`'s that are non-dominated with respect to each other.
/// This is useful for multi-objective optimization problems where the goal is to find
/// the best solutions that are not dominated by any other solution.
/// This results in what is called the Pareto front.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Front<T>
where
    T: Scored,
{
    values: Vec<Arc<T>>,
    range: Range<usize>,
    objective: Objective,
}

impl<T> Front<T>
where
    T: Scored,
{
    pub fn new(range: Range<usize>, objective: Objective) -> Self {
        Front {
            values: Vec::new(),
            range,
            objective,
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
                if self.dom_cmp(existing_val.as_ref(), new_member) == Ordering::Greater || equals {
                    // If an existing value dominates the new value, return false
                    is_dominated = false;
                    break;
                } else if self.dom_cmp(new_member, existing_val.as_ref()) == Ordering::Greater {
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

    fn dom_cmp(&self, one: &T, two: &T) -> Ordering {
        let one_score = one.score();
        let two_score = two.score();

        if one_score.is_none() || two_score.is_none() {
            return Ordering::Equal;
        }

        if let (Some(one), Some(two)) = (one_score, two_score) {
            if pareto::dominance(one, two, &self.objective) {
                return Ordering::Greater;
            } else if pareto::dominance(two, one, &self.objective) {
                return Ordering::Less;
            }
        }

        Ordering::Equal
    }

    fn filter(&mut self) {
        let values = self
            .values
            .iter()
            .filter_map(|s| s.score())
            .collect::<Vec<_>>();
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
    T: Scored,
{
    fn default() -> Self {
        Front::new(0..0, Objective::Single(Optimize::Minimize))
    }
}
