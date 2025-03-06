use crate::objectives::{Objective, pareto};
use std::{
    cmp::{self, Ordering},
    ops::Range,
    sync::Arc,
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
    ord: Arc<dyn Fn(&T, &T) -> Ordering>,
    range: Range<usize>,
    objective: Objective,
}

impl<T> Front<T>
where
    T: PartialEq + Clone + AsRef<[f32]>,
{
    pub fn new<F>(range: Range<usize>, objective: Objective, comp: F) -> Self
    where
        F: Fn(&T, &T) -> Ordering + 'static,
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

    pub fn update_front(&mut self, scores: &[T]) -> usize {
        let mut count = 0;
        for value in scores {
            if self.add(value) {
                count += 1;
            }
        }

        if self.values.len() > self.range.end {
            self.filter();
        }

        count
    }

    pub fn add(&mut self, score: &T) -> bool {
        let mut to_remove = Vec::new();
        let mut is_dominated = false;
        let mut remove_duplicates = false;

        for existing_score in self.values.iter() {
            if (self.ord)(score, existing_score) == Ordering::Greater {
                to_remove.push(Arc::clone(existing_score));
            } else if (self.ord)(existing_score, score) == Ordering::Greater
                || (&*(*existing_score)).as_ref() == score.as_ref()
            {
                is_dominated = true;
                remove_duplicates = true;
                break;
            }
        }

        if remove_duplicates {
            self.values.retain(|x| !to_remove.contains(x));
        }

        if !is_dominated {
            self.values.retain(|x| !to_remove.contains(x));
            self.values.push(Arc::new(score.clone()));
            return true;
        }

        false
    }

    fn filter(&mut self) {
        let values = self
            .values
            .iter()
            .map(|s| (*(*s).as_ref()).as_ref())
            .collect::<Vec<_>>();
        let crowding_distances = pareto::crowding_distance(&values, &self.objective);

        let mut enumerated = crowding_distances.iter().enumerate().collect::<Vec<_>>();

        enumerated.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(cmp::Ordering::Equal));

        self.values = enumerated
            .iter()
            .take(self.range.end)
            .map(|(i, _)| Arc::clone(&self.values[*i]))
            .collect::<Vec<Arc<T>>>();
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct Front {
//     scores: Vec<Vec<f32>>,
//     range: Range<usize>,
//     objective: Objective,
// }

// impl Front {
//     pub fn new(range: Range<usize>, objective: Objective) -> Self {
//         Front {
//             scores: Vec::new(),
//             range,
//             objective,
//         }
//     }

//     pub fn scores(&self) -> &[Vec<f32>] {
//         &self.scores``
//     }

//     /// Update the front with a new set of scores. This will add the scores to the front
//     /// and filter out any dominated scores. If the front exceeds the maximum size, it will
//     /// filter out the least crowded scores.
//     pub fn update_front(&mut self, scores: &[Vec<f32>]) {
//         for score in scores {
//             self.add(score);
//         }

//         if self.scores.len() > self.range.end {
//             self.filter();
//         }
//     }

//     fn add(&mut self, score: &Vec<f32>) {
//         let mut to_remove = Vec::new();
//         let mut is_dominated = false;
//         let mut remove_duplicates = false;

//         for existing_score in &self.scores {
//             if pareto::dominance(score, existing_score, &self.objective) {
//                 to_remove.push(existing_score.clone());
//             } else if pareto::dominance(existing_score, score, &self.objective)
//                 || existing_score == score
//             {
//                 is_dominated = true;
//                 remove_duplicates = true;
//                 break;
//             }
//         }

//         if remove_duplicates {
//             self.scores.retain(|x| !to_remove.contains(x));
//         }

//         if !is_dominated {
//             self.scores.retain(|x| !to_remove.contains(x));
//             self.scores.push(score.clone());
//         }
//     }

//     fn filter(&mut self) {
//         let scores = self.scores.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
//         let crowding_distances = pareto::crowding_distance(&scores, &self.objective);

//         let mut enumerated = crowding_distances.iter().enumerate().collect::<Vec<_>>();

//         enumerated.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(cmp::Ordering::Equal));

//         self.scores = enumerated
//             .iter()
//             .take(self.range.end)
//             .map(|(i, _)| self.scores[*i].clone())
//             .collect::<Vec<Vec<f32>>>();
//     }
// }
