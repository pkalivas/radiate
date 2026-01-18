use crate::objectives::{Objective, Scored, pareto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{cmp::Ordering, hash::Hash, ops::Range, sync::Arc};

const DEFAULT_ENTROPY_BINS: usize = 20;
const EPSILON: f32 = 1e-10;

#[derive(Clone, Default)]
struct FrontScratch {
    remove: Vec<usize>,
    keep_idx: Vec<usize>,
    scores: Vec<f32>,
    dist: Vec<f32>,
    order: Vec<usize>,
}

pub struct FrontAddResult {
    pub added_count: usize,
    pub removed_count: usize,
    pub comparisons: usize,
    pub filter_count: usize,
    pub size: usize,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Front<T>
where
    T: Scored,
{
    values: Vec<Arc<T>>,
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

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn objective(&self) -> Objective {
        self.objective.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn values(&self) -> &[Arc<T>] {
        &self.values
    }

    pub fn crowding_distance(&mut self) -> Option<Vec<f32>> {
        self.ensure_score_matrix()?;
        let (n, _) = self.score_dims()?;
        self.crowding_distance_in_place(n);

        Some(self.scratch.dist[..n].to_vec())
    }

    pub fn entropy(&mut self) -> Option<f32> {
        self.ensure_score_matrix()?;
        let (n, m) = self.score_dims()?;

        Some(entropy_flat(
            &self.scratch.scores,
            n,
            m,
            DEFAULT_ENTROPY_BINS,
        ))
    }

    pub fn add_all(&mut self, items: Vec<T>) -> FrontAddResult
    where
        T: Eq + Hash + Clone + Send + Sync + 'static,
    {
        let mut added_count = 0;
        let mut removed_count = 0;
        let mut comparisons = 0;
        let mut filter_count = 0;

        for new_member in items.into_iter() {
            self.scratch.remove.clear();

            // Decide accept/reject without mutating self.values
            let mut accept = true;

            for (idx, existing) in self.values.iter().enumerate() {
                if existing.as_ref() == &new_member {
                    accept = false;
                    break;
                }

                // dominance checks
                match self.dom_cmp(existing.as_ref(), &new_member) {
                    Ordering::Greater => {
                        // existing dominates new -> reject
                        accept = false;
                        comparisons += 1;
                        break;
                    }
                    Ordering::Less => {
                        // new dominates existing -> mark for removal
                        self.scratch.remove.push(idx);
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
            if !self.scratch.remove.is_empty() {
                self.scratch.remove.sort_unstable();
                self.scratch.remove.dedup();

                for &idx in self.scratch.remove.iter().rev() {
                    self.values.swap_remove(idx);
                    removed_count += 1;
                }
            }

            self.values.push(Arc::new(new_member));
            added_count += 1;

            // Filter if we exceed max
            if self.values.len() > self.range.end {
                self.fast_filter();
                filter_count += 1;
            }

            // Invalidate the cached score matrix
            self.scratch.scores.clear();
        }

        FrontAddResult {
            added_count,
            removed_count,
            comparisons,
            filter_count,
            size: self.values.len(),
        }
    }

    #[inline]
    fn dom_cmp(&self, one: &T, two: &T) -> Ordering {
        let one_score = one.score();
        let two_score = two.score();

        if one_score.is_none() || two_score.is_none() {
            return Ordering::Equal;
        }

        let (a, b) = (one_score.unwrap(), two_score.unwrap());

        if pareto::dominance(a, b, &self.objective) {
            Ordering::Greater
        } else if pareto::dominance(b, a, &self.objective) {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }

    fn fast_filter(&mut self) {
        let keep = self.range.start.min(self.values.len());
        if keep == 0 || self.values.len() <= keep {
            return;
        }

        // Build score matrix + crowding distances into scratch
        if self.ensure_score_matrix().is_none() {
            return;
        }

        let (n, _m) = match self.score_dims() {
            Some(x) => x,
            None => return,
        };

        self.crowding_distance_in_place(n);

        // Pick top `keep` by crowding distance without sorting all n.
        self.scratch.keep_idx.clear();
        self.scratch.keep_idx.extend(0..n);

        let dist = &self.scratch.dist;

        // Partition so that [0..keep] are the best (in any order)
        self.scratch
            .keep_idx
            .select_nth_unstable_by(keep, |&i, &j| {
                dist[j].partial_cmp(&dist[i]).unwrap_or(Ordering::Equal)
            });

        self.scratch.keep_idx.truncate(keep);

        let mut new_values = Vec::with_capacity(keep);
        for &i in self.scratch.keep_idx.iter() {
            new_values.push(Arc::clone(&self.values[i]));
        }

        self.values = new_values;
        self.scratch.scores.clear();
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

    fn ensure_score_matrix(&mut self) -> Option<()> {
        let (n, m) = self.score_dims()?;

        if m == 0 {
            return None;
        }

        // If already built and size matches, keep it.
        if self.scratch.scores.len() == n * m {
            return Some(());
        }

        self.scratch.scores.resize(n * m, 0.0);
        for (i, v) in self.values.iter().enumerate() {
            let s = v.score()?;
            if s.len() != m {
                return None;
            }

            let row = &mut self.scratch.scores[i * m..i * m + m];
            row.copy_from_slice(s.as_slice());
        }

        Some(())
    }

    fn crowding_distance_in_place(&mut self, n: usize) {
        let (_, m) = match self.score_dims() {
            Some(x) => x,
            None => return,
        };

        if n == 0 || m == 0 {
            return;
        }

        self.scratch.dist.clear();
        self.scratch.dist.resize(n, 0.0);

        self.scratch.order.clear();
        self.scratch.order.extend(0..n);

        for dim in 0..m {
            let scores = &self.scratch.scores;
            self.scratch.order.sort_unstable_by(|&i, &j| {
                let a = scores[i * m + dim];
                let b = scores[j * m + dim];
                a.partial_cmp(&b).unwrap_or(Ordering::Equal)
            });

            let first_idx = self.scratch.order[0];
            let last_idx = self.scratch.order[n - 1];
            let min = self.scratch.scores[first_idx * m..first_idx * m + m][dim];
            let max = self.scratch.scores[last_idx * m..last_idx * m + m][dim];
            let range = max - min;

            if !range.is_finite() || range == 0.0 {
                continue;
            }

            self.scratch.dist[self.scratch.order[0]] = f32::INFINITY;
            self.scratch.dist[self.scratch.order[n - 1]] = f32::INFINITY;

            for k in 1..(n - 1) {
                let prev_idx = self.scratch.order[k - 1];
                let next_idx = self.scratch.order[k + 1];
                let prev = self.scratch.scores[prev_idx * m..prev_idx * m + m][dim];
                let next = self.scratch.scores[next_idx * m..next_idx * m + m][dim];

                let contrib = (next - prev).abs() / range;
                self.scratch.dist[self.scratch.order[k]] += contrib;
            }
        }
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

/// Calculate the Shannon entropy of a set of scores in multi-dimensional space.
/// The scores are discretized into a grid of bins, and the entropy is computed
/// based on the distribution of scores across these bins. Higher entropy indicates
/// a more diverse set of scores. This can be interpreted as a measure of how well
/// the solutions are spread out in the objective space.
///
/// It works by:
/// 1. Determining the min and max values for each objective dimension.
/// 2. Mapping each score to a discrete bin index based on its normalized position
///    within the min-max range for each dimension.
/// 3. Counting the number of scores in each bin (cell).
/// 4. Calculating the probabilities of each occupied bin and computing the
///    Shannon entropy using these probabilities.
/// 5. Optionally normalizing the entropy by the maximum possible entropy given
///    the number of occupied bins and total scores.
fn entropy_flat(scores: &[f32], n: usize, m: usize, bins_per_dim: usize) -> f32 {
    if n == 0 || m == 0 || bins_per_dim == 0 {
        return 0.0;
    }

    // mins/maxs per dim
    let mut mins = vec![f32::INFINITY; m];
    let mut maxs = vec![f32::NEG_INFINITY; m];

    for i in 0..n {
        let row = &scores[i * m..i * m + m];
        for d in 0..m {
            let x = row[d];
            if x < mins[d] {
                mins[d] = x;
            }
            if x > maxs[d] {
                maxs[d] = x;
            }
        }
    }

    for d in 0..m {
        if (maxs[d] - mins[d]).abs() < EPSILON {
            maxs[d] = mins[d] + 1.0;
        }
    }

    let mut cell_counts: HashMap<Vec<u8>, usize> = HashMap::new();

    for i in 0..n {
        let row = &scores[i * m..i * m + m];
        let mut cell = Vec::with_capacity(m);

        for d in 0..m {
            let norm = (row[d] - mins[d]) / (maxs[d] - mins[d]); // [0,1]
            let mut idx = (norm * bins_per_dim as f32).floor() as i32;
            if idx < 0 {
                idx = 0;
            }
            if idx >= bins_per_dim as i32 {
                idx = bins_per_dim as i32 - 1;
            }
            cell.push(idx as u8);
        }

        *cell_counts.entry(cell).or_insert(0) += 1;
    }

    let n_f = n as f32;
    let mut h = 0.0_f32;
    for &count in cell_counts.values() {
        let p = count as f32 / n_f;
        if p > 0.0 {
            h -= p * p.ln();
        }
    }

    let k = cell_counts.len().min(n);
    if k > 1 { h / (k as f32).ln() } else { 0.0 }
}
