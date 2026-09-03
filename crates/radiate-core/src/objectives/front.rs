use crate::objectives::{Objective, Scored, pareto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, ops::Range};

const DEFAULT_ENTROPY_BINS: usize = 20;

#[derive(Debug)]
pub struct FrontAddResult {
    pub added_count: usize,
    pub removed_count: usize,
    pub comparisons: usize,
    pub filter_count: usize,
    pub size: usize,
}

#[derive(Clone, Default)]
struct FrontScratch {
    remove_buff: Vec<usize>,
    index_buff: Vec<usize>,
    crowding_buff: Vec<f32>,
    filter_buff: Vec<bool>,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Front<T>
where
    T: Scored,
{
    values: Vec<T>,
    range: Range<usize>,
    objective: Objective,

    #[cfg_attr(feature = "serde", serde(skip))]
    scratch: FrontScratch,
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
            scratch: FrontScratch::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn objective(&self) -> Objective {
        self.objective.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn crowding_distance(&mut self) -> Option<&[f32]> {
        let scores = self
            .values
            .iter()
            .filter_map(|v| v.score())
            .collect::<Vec<_>>();

        if scores.is_empty() {
            return None;
        }

        self.scratch.crowding_buff.clear();
        self.scratch.crowding_buff.resize(scores.len(), 0.0);

        pareto::buffered_crowding_distance(&scores, &mut self.scratch.crowding_buff);

        Some(&self.scratch.crowding_buff[..])
    }

    pub fn entropy(&mut self) -> Option<f32> {
        let scores = self
            .values
            .iter()
            .filter_map(|v| v.score())
            .collect::<Vec<_>>();

        if scores.is_empty() {
            return None;
        }

        Some(pareto::entropy(scores.as_slice(), DEFAULT_ENTROPY_BINS))
    }

    pub fn try_add_all<'a>(&mut self, items: impl Iterator<Item = &'a T>) -> FrontAddResult
    where
        T: Eq + Clone + 'static,
    {
        let mut added_count = 0;
        let mut removed_count = 0;
        let mut comparisons = 0;
        let mut filter_count = 0;

        for new_member in items.into_iter() {
            self.scratch.remove_buff.clear();

            // Decide accept/reject without mutating self.values
            let mut accept = true;

            for (idx, existing) in self.values.iter().enumerate() {
                if existing == new_member {
                    accept = false;
                    break;
                }

                match self.dom_cmp(existing, new_member) {
                    Ordering::Greater => {
                        // existing dominates new -> reject
                        accept = false;
                        comparisons += 1;
                        break;
                    }
                    Ordering::Less => {
                        // new dominates existing -> mark for removal
                        self.scratch.remove_buff.push(idx);
                        comparisons += 1;
                    }
                    Ordering::Equal => comparisons += 1,
                }
            }

            if !accept {
                continue;
            }

            // Remove dominated existing values efficiently (swap_remove).
            // Need stable removal: remove in descending index order.
            if !self.scratch.remove_buff.is_empty() {
                self.scratch.remove_buff.sort_unstable();
                self.scratch.remove_buff.dedup();

                removed_count += self.scratch.remove_buff.len();

                for &idx in self.scratch.remove_buff.iter().rev() {
                    self.values.swap_remove(idx);
                }
            }

            self.values.push(new_member.clone());
            added_count += 1;

            // Filter if we exceed max
            if self.values.len() > self.range.end {
                self.fast_filter();
                filter_count += 1;
            }
        }

        FrontAddResult {
            added_count,
            removed_count,
            comparisons,
            filter_count,
            size: self.values.len(),
        }
    }

    /// Remove points with crowding distance in the top `trim` fraction.
    /// Example: trim=0.02 removes the top 2% most isolated points.
    #[inline]
    pub fn remove_outliers(&mut self, trim: f32) -> Option<usize> {
        if self.values.len() < 4 {
            return None;
        }

        let trim = trim.clamp(0.0, 0.5);
        if trim == 0.0 {
            return None;
        }

        let (n, _) = self.score_dims()?;

        let drop = ((n as f32) * trim).floor() as usize;
        if drop == 0 {
            return None;
        }

        let scores = self
            .values
            .iter()
            .filter_map(|v| v.score())
            .collect::<Vec<_>>();

        self.scratch.crowding_buff.clear();
        self.scratch.crowding_buff.resize(scores.len(), 0.0);

        self.scratch.index_buff.clear();
        self.scratch.index_buff.extend(0..scores.len());

        pareto::buffered_crowding_distance(&scores, &mut self.scratch.crowding_buff);

        self.scratch.index_buff.sort_unstable_by(|&i, &j| {
            self.scratch.crowding_buff[j]
                .partial_cmp(&self.scratch.crowding_buff[i])
                .unwrap_or(Ordering::Equal)
        });

        self.scratch.index_buff.truncate(drop);
        self.scratch.index_buff.sort_unstable();
        self.scratch.index_buff.dedup();

        let removed = self.scratch.index_buff.len();
        for &idx in self.scratch.index_buff.iter().rev() {
            self.values.swap_remove(idx);
        }

        Some(removed)
    }

    pub fn fronts(&mut self) -> Vec<Front<T>>
    where
        T: Clone + Eq + Send + Sync + 'static,
    {
        let mut fronts: Vec<Front<T>> = Vec::new();
        for member in self.values.iter() {
            let mut updated = false;

            for front in fronts.iter_mut() {
                let result = front.try_add_all(std::iter::once(member));

                if result.added_count > 0 {
                    updated = true;
                    break;
                }
            }

            if !updated {
                let mut new_front = Front::new(self.range.clone(), self.objective.clone());
                new_front.try_add_all(std::iter::once(member));
                fronts.push(new_front);
            }
        }

        fronts
    }

    fn fast_filter(&mut self) {
        let keep = self.range.start.min(self.values.len());
        if keep == 0 || self.values.len() <= keep {
            return;
        }

        let scores = self
            .values
            .iter()
            .filter_map(|v| v.score())
            .collect::<Vec<_>>();

        self.scratch.crowding_buff.clear();
        self.scratch.crowding_buff.resize(scores.len(), 0.0);

        self.scratch.index_buff.clear();
        self.scratch.index_buff.extend(0..scores.len());

        pareto::buffered_crowding_distance(&scores, &mut self.scratch.crowding_buff);

        self.scratch
            .index_buff
            .select_nth_unstable_by(keep, |&a, &b| {
                self.scratch.crowding_buff[b]
                    .partial_cmp(&self.scratch.crowding_buff[a])
                    .unwrap_or(Ordering::Equal)
            });
        self.scratch.index_buff.truncate(keep);

        self.retain_indices();
    }

    #[inline]
    fn dom_cmp(&self, one: &T, two: &T) -> Ordering {
        let one_score = one.score();
        let two_score = two.score();

        if one_score.is_none() || two_score.is_none() {
            return Ordering::Equal;
        }

        if let Some((a, b)) = one_score.zip(two_score) {
            if pareto::dominance(a, b, &self.objective) {
                return Ordering::Greater;
            } else if pareto::dominance(b, a, &self.objective) {
                return Ordering::Less;
            }
        }
        Ordering::Equal
    }

    /// Keep only the elements at `indices`, in `self.values`' current order.
    /// Reuses `scratch.keep_true` as scan-line scratch instead of allocating
    /// a fresh mask on every call.
    fn retain_indices(&mut self) {
        self.scratch.filter_buff.clear();
        self.scratch.filter_buff.resize(self.values.len(), false);

        for &idx in self.scratch.index_buff.iter() {
            self.scratch.filter_buff[idx] = true;
        }

        // Bind disjoint fields locally so the borrow checker sees this as
        // two separate borrows of `self.values` and `self.scratch.keep_true`
        // rather than one borrow of `self`.
        let values = &mut self.values;
        let keep_true = &self.scratch.filter_buff;

        let mut idx = 0;
        values.retain(|_| {
            let retain = keep_true[idx];
            idx += 1;
            retain
        });
    }

    #[inline]
    fn score_dims(&self) -> Option<(usize, usize)> {
        let n = self.values.len();

        if n == 0 {
            return None;
        }

        let first = self.values.iter().find_map(|v| v.score())?;
        Some((n, first.len()))
    }
}

impl<T> Default for Front<T>
where
    T: Scored,
{
    fn default() -> Self {
        Front::new(0..0, Objective::default())
    }
}
