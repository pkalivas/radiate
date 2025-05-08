use crate::{
    objectives::{Objective, pareto},
    thread_pool::{ThreadPool, WaitGroup},
};
use std::{
    cmp::Ordering,
    collections::HashSet,
    hash::Hash,
    ops::Range,
    sync::{Arc, RwLock},
};

/// A front is a collection of scores that are non-dominated with respect to each other.
/// This is useful for multi-objective optimization problems where the goal is to find
/// the best solutions that are not dominated by any other solution.
/// This results in what is called the Pareto front.
#[derive(Clone)]
pub struct Front<T>
where
    T: PartialEq + Clone + AsRef<[f32]>,
{
    values: Vec<Arc<T>>,
    ord: Arc<dyn Fn(&T, &T) -> Ordering + Send + Sync>,
    range: Range<usize>,
    objective: Objective,
    thread_pool: Arc<ThreadPool>,
}

impl<T> Front<T>
where
    T: PartialEq + Clone + AsRef<[f32]>,
{
    pub fn new<F>(
        range: Range<usize>,
        objective: Objective,
        thread_pool: Arc<ThreadPool>,
        comp: F,
    ) -> Self
    where
        F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
    {
        Front {
            values: Vec::new(),
            range,
            objective,
            ord: Arc::new(comp),
            thread_pool,
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
        let wg = WaitGroup::new();
        let ord = Arc::clone(&self.ord);
        let values = Arc::new(RwLock::new(self.values.clone()));
        let dominating_values = Arc::new(RwLock::new(vec![false; items.len()]));
        let remove_values = Arc::new(RwLock::new(HashSet::new()));
        let values_to_add = Arc::new(RwLock::new(Vec::new()));

        for (idx, member) in items.iter().enumerate() {
            let ord_clone = Arc::clone(&ord);
            let values_clone = Arc::clone(&values);
            let doms_vector = Arc::clone(&dominating_values);
            let remove_vector = Arc::clone(&remove_values);
            let new_member = member.clone();
            let values_to_add = Arc::clone(&values_to_add);

            self.thread_pool.group_submit(&wg, move || {
                let mut is_dominated = true;

                for existing_val in values_clone.read().unwrap().iter() {
                    if (ord_clone)(existing_val, &new_member) == Ordering::Greater {
                        // If an existing value dominates the new value, return false
                        is_dominated = false;
                        break;
                    } else if (ord_clone)(&new_member, existing_val) == Ordering::Greater {
                        // If the new value dominates an existing value, continue checking
                        // to_remove.push(Arc::clone(existing_val));
                        remove_vector.write().unwrap().insert(existing_val.clone());
                        continue;
                    } else if &new_member == existing_val.as_ref() {
                        // If they are equal, we consider it dominated
                        is_dominated = false;
                        break;
                    }
                }

                if is_dominated {
                    doms_vector.write().unwrap().get_mut(idx).map(|v| *v = true);
                    let mut writer = values_to_add.write().unwrap();
                    writer.push(new_member);
                }
            });
        }

        let count = wg.wait();

        self.values
            .retain(|x| !remove_values.read().unwrap().contains(x));
        self.values
            .extend(values_to_add.write().unwrap().drain(..).map(Arc::new));

        if self.values.len() > self.range.end {
            self.filter();
        }

        count
    }

    pub fn filter(&mut self) {
        let values = self.values.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
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
