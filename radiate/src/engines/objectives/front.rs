use super::Scored;
use crate::objectives::{Objective, pareto};
use std::{cmp::Ordering, ops::Range, sync::Arc};

/// A front is a collection of scores that are non-dominated with respect to each other.
/// This is useful for multi-objective optimization problems where the goal is to find
/// the best solutions that are not dominated by any other solution.
/// This results in what is called the Pareto front.
#[derive(Clone)]
pub struct Front<T>
where
    T: PartialEq + Clone + Scored,
{
    values: Vec<Arc<T>>,
    ord: Arc<dyn Fn(&T, &T) -> Ordering + Send + Sync>,
    range: Range<usize>,
    objective: Objective,
}

impl<T> Front<T>
where
    T: PartialEq + Clone + Scored,
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

    pub fn values(&self) -> &[Arc<T>] {
        &self.values
    }

    pub fn update_front<V>(&mut self, values: &[V]) -> usize
    where
        V: AsRef<T>,
    {
        let mut count = 0;
        for value in values {
            if self.add(value.as_ref()) {
                count += 1;
            }
        }

        if self.values.len() > self.range.end {
            self.filter();
        }

        count
    }

    pub fn dominates(&self, value: &T) -> (bool, Vec<Arc<T>>) {
        let mut to_remove = Vec::new();
        for existing_val in self.values.iter() {
            if (self.ord)(existing_val.as_ref(), value) == Ordering::Greater {
                // If an existing value dominates the new value, return false
                return (false, to_remove);
            } else if (self.ord)(value, existing_val.as_ref()) == Ordering::Greater {
                // If the new value dominates an existing value, continue checking
                to_remove.push(Arc::clone(existing_val));
                continue;
            } else if value == existing_val.as_ref() {
                // If they are equal, we consider it dominated
                return (false, to_remove);
            }
        }

        (true, to_remove)
    }

    pub fn clean(&mut self, new_values: Vec<&T>, to_remove: &[Arc<T>]) {
        self.values.retain(|x| !to_remove.contains(x));

        for new_val in new_values {
            self.values.push(Arc::new(new_val.clone()));
        }

        if self.values.len() > self.range.end {
            self.filter();
        }
    }

    pub fn add(&mut self, value: &T) -> bool {
        let mut to_remove = Vec::new();
        let mut is_dominated = false;

        for existing_val in self.values.iter() {
            if (self.ord)(value, existing_val) == Ordering::Greater {
                to_remove.push(Arc::clone(existing_val));
            } else if (self.ord)(existing_val, value) == Ordering::Greater
                || value == existing_val.as_ref()
            {
                is_dominated = true;
                break;
            }
        }

        if !is_dominated {
            self.values.retain(|x| !to_remove.contains(x));
            self.values.push(Arc::new(value.clone()));
            return true;
        }

        false
    }

    fn filter(&mut self) {
        let values = self
            .values
            .iter()
            .map(|s| s.score().unwrap())
            .collect::<Vec<_>>();
        let crowding_distances = pareto::crowding_distance(&values, &self.objective);

        let mut enumerated = crowding_distances.iter().enumerate().collect::<Vec<_>>();

        enumerated.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(Ordering::Equal));

        self.values = enumerated
            .iter()
            .take(self.range.end)
            .map(|(i, _)| Arc::clone(&self.values[*i]))
            .collect::<Vec<Arc<T>>>();
    }
}
