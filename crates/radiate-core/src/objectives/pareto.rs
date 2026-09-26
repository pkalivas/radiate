//! Multi-objective optimization utilities, including Pareto front calculation,
//! non-dominated sorting, crowding distance, and entropy measures.
//! These are essential for evolutionary algorithms that need to handle
//! multiple conflicting objectives.

use crate::objectives::{Objective, Optimize};
use radiate_utils::Float;
use std::collections::HashMap;

/// A small constant to avoid division by zero and ensure non-zero weights.
const EPSILON: f32 = 1e-6;

/// Calculate the crowding distance for each score in a population.
///
/// The crowding distance is a measure of how close a score is to its neighbors
/// in the objective space. Scores with a higher crowding distance are more
/// desirable because they are more spread out. This is useful for selecting
/// diverse solutions in a multi-objective optimization problem and is a
/// key component of the NSGA-II algorithm.
///
/// For each objective dimension:
/// - Sort individuals by that objective
/// - Boundary points get +∞ distance (always preferred)
/// - Interior points get normalized distance contribution:
///
/// ```text
/// (f_{i+1} - f_{i-1}) / (f_max - f_min)
/// ```
#[inline]
pub fn crowding_distance<T: AsRef<[f32]>>(scores: &[T]) -> Vec<f32> {
    let mut buffer = vec![0.0; scores.len()];

    if !ensure_buffer_len(scores, &mut buffer) {
        return buffer;
    }

    buffered_crowding_distance(scores, &mut buffer);
    buffer
}

#[inline]
pub fn buffered_crowding_distance<T: AsRef<[f32]>>(scores: &[T], buffer: &mut [f32]) {
    if !ensure_buffer_len(scores, buffer) {
        return;
    }

    let n = scores.len();
    if buffer.len() < n {
        panic!(
            "Buffer length {} is less than number of scores {}",
            buffer.len(),
            n
        );
    }

    let mut indices = (0..n).collect::<Vec<usize>>();
    let m = scores[0].as_ref().len();
    for dim in 0..m {
        indices.sort_unstable_by(|&i, &j| {
            scores[i].as_ref()[dim]
                .partial_cmp(&scores[j].as_ref()[dim])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let min = scores[indices[0]].as_ref()[dim];
        let max = scores[indices[n - 1]].as_ref()[dim];
        let range = max - min;

        if !range.is_finite() || range == 0.0 {
            continue;
        }

        buffer[indices[0]] = f32::INFINITY;
        buffer[indices[n - 1]] = f32::INFINITY;

        for k in 1..(n - 1) {
            let prev = scores[indices[k - 1]].as_ref()[dim];
            let next = scores[indices[k + 1]].as_ref()[dim];
            let contrib = (next - prev).abs() / range;
            buffer[indices[k]] += contrib;
        }
    }
}

#[inline]
pub fn front_crowding_distance<T: AsRef<[f32]>>(scores: &[T], ranks: &[usize]) -> Vec<f32> {
    let mut distances = vec![0.0; scores.len()];

    for front in fronts_from_ranks(ranks) {
        let front_scores = front
            .iter()
            .map(|&i| scores[i].as_ref())
            .collect::<Vec<_>>();
        for (&idx, distance) in front.iter().zip(crowding_distance(&front_scores)) {
            distances[idx] = distance;
        }
    }

    distances
}

#[inline]
pub fn fronts_from_ranks(ranks: &[usize]) -> Vec<Vec<usize>> {
    if ranks.is_empty() {
        return Vec::new();
    }

    let max_rank = *ranks.iter().max().unwrap_or(&0);
    let mut fronts = vec![Vec::<usize>::new(); max_rank + 1];

    for (idx, &rank) in ranks.iter().enumerate() {
        fronts[rank].push(idx);
    }

    while fronts.last().is_some_and(|front| front.is_empty()) {
        fronts.pop();
    }

    fronts
}

#[inline]
pub fn non_dominated<T: AsRef<[f32]>>(population: &[T], objective: &Objective) -> Vec<usize> {
    let n = population.len();
    if n == 0 {
        return Vec::new();
    }

    let mut dominated_counts = vec![0usize; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let a = &population[i];
            let b = &population[j];

            if dominance(a, b, objective) {
                dominated_counts[j] += 1;
            } else if dominance(b, a, objective) {
                dominated_counts[i] += 1;
            }
        }
    }

    let mut nd = Vec::new();
    for (i, counts) in dominated_counts.iter().enumerate() {
        if *counts == 0 {
            nd.push(i);
        }
    }

    nd
}

/// Rank the population based on the NSGA-II algorithm. This assigns a rank to each
/// individual in the population based on their dominance relationships with other
/// individuals in the population. The result is a vector of ranks, where the rank
///   of the individual at index `i` is `ranks[i]`.
#[inline]
pub fn rank<T: AsRef<[f32]>>(population: &[T], objective: &Objective) -> Vec<usize> {
    let n = population.len();
    if n == 0 {
        return Vec::new();
    }

    let mut dominated_counts = vec![0usize; n];
    let mut dominates: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut current_front: Vec<usize> = Vec::new();

    // Build dominates lists + dominated counts in one pass (no NxN matrix).
    for i in 0..n {
        for j in (i + 1)..n {
            let a = &population[i];
            let b = &population[j];

            if dominance(a, b, objective) {
                dominates[i].push(j);
                dominated_counts[j] += 1;
            } else if dominance(b, a, objective) {
                dominates[j].push(i);
                dominated_counts[i] += 1;
            }
        }
    }

    // First front
    for (i, count) in dominated_counts.iter().enumerate() {
        if *count == 0 {
            current_front.push(i);
        }
    }

    let mut ranks = vec![0usize; n];
    let mut front_idx = 0usize;

    while !current_front.is_empty() {
        let mut next_front = Vec::new();

        for &p in &current_front {
            ranks[p] = front_idx;

            for &q in &dominates[p] {
                dominated_counts[q] -= 1;
                if dominated_counts[q] == 0 {
                    next_front.push(q);
                }
            }
        }

        front_idx += 1;
        current_front = next_front;
    }

    ranks
}

/// Combine NSGA-II rank and crowding distance into a single weight in (0, 1].
///
/// - Lower rank (better front) => higher weight
/// - Higher crowding distance  => higher weight
///
/// This weight vector combines both rank and crowding distance to prioritize
/// individuals that are both in better fronts and more diverse within those fronts. Selection
/// algorithms not specifically designed for multi-objective optimization can use these weights
/// as fitness values to guide selection towards a well-distributed Pareto front.
///
/// It follows the approach outlined in the paper [A Fast and Elitist Multiobjective Genetic
/// Algorithm: NSGA-II](https://sci2s.ugr.es/sites/default/files/files/Teaching/OtherPostGraduateCourses/Metaheuristicas/Deb_NSGAII.pdf) by
/// K. Deb, A. Pratap, S. Agarwal, and T. Meyarivan
/// pp. 182-197, Apr. 2002, doi: 10.1109/4235.996017.
///
/// We follow these steps:
/// 1. Compute ranks using the `rank` function (lower is better)
/// 2. Compute crowding distances using the `crowding_distance` function (higher is better).
/// 3. Normalize ranks to [0, 1], where 1 = best front.
/// 4. Normalize crowding distances to [0, 1], where 1 = most isolated.
/// 5. Combine the two normalized values multiplicatively to get the final weight.
#[inline]
pub fn weights<T: AsRef<[f32]>>(scores: &[T], objective: &Objective) -> Vec<f32> {
    let n = scores.len();
    if n == 0 {
        return Vec::new();
    }

    let ranks = rank(scores, objective);
    let distances = crowding_distance(scores);

    let max_rank = *ranks.iter().max().unwrap_or(&0) as f32;

    let rank_weight = ranks
        .iter()
        .map(|r| {
            if max_rank == 0.0 {
                1.0
            } else {
                1.0 - (*r as f32 / max_rank)
            }
        })
        .collect::<Vec<f32>>();

    let finite_max = distances
        .iter()
        .cloned()
        .filter(|d| d.is_finite())
        .fold(0.0f32, f32::max);

    let crowd_weight = distances
        .iter()
        .map(|d| {
            if !d.is_finite() || finite_max == 0.0 {
                1.0
            } else {
                *d / finite_max
            }
        })
        .collect::<Vec<f32>>();

    rank_weight
        .into_iter()
        .zip(crowd_weight)
        .map(|(r, c)| (r + EPSILON).max(0.0) * (c + EPSILON).max(0.0))
        .collect()
}

// Determine if one score dominates another score. A score `a` dominates a score `b`
// if it is better in every objective and at least one objective is strictly better.
pub fn dominance<K: PartialOrd, T: AsRef<[K]>>(
    score_a: &T,
    score_b: &T,
    objective: &Objective,
) -> bool {
    let mut better_in_any = false;
    match objective {
        Objective::Single(opt) => {
            for (a, b) in score_a.as_ref().iter().zip(score_b.as_ref().iter()) {
                if opt == &Optimize::Minimize {
                    if a > b {
                        return false;
                    }
                    if a < b {
                        better_in_any = true;
                    }
                } else {
                    if a < b {
                        return false;
                    }
                    if a > b {
                        better_in_any = true;
                    }
                }
            }
        }
        Objective::Multi(opts) => {
            for ((a, b), opt) in score_a.as_ref().iter().zip(score_b.as_ref()).zip(opts) {
                if opt == &Optimize::Minimize {
                    if a > b {
                        return false;
                    }
                    if a < b {
                        better_in_any = true;
                    }
                } else {
                    if a < b {
                        return false;
                    }
                    if a > b {
                        better_in_any = true;
                    }
                }
            }
        }
    }

    better_in_any
}

/// Calculate the Pareto front of a set of scores. The Pareto front is the set of
/// scores that are not dominated by any other score in the set. This is useful
/// for selecting the best solutions in a multi-objective optimization problem.
pub fn pareto_front<K: PartialOrd, T: AsRef<[K]> + Clone>(
    values: &[T],
    objective: &Objective,
) -> Vec<T> {
    let mut front = Vec::new();
    for score in values {
        let mut dominated = false;
        for other in values {
            if dominance(other, score, objective) {
                dominated = true;
                break;
            }
        }
        if !dominated {
            front.push(score.clone());
        }
    }

    front
}

/// Calculate the hypervolume indicator of a set of scores with respect to a `reference` point.
///
/// The hypervolume is the size of the objective-space region that is dominated by at least one
/// score and that in turn dominates the reference point. Larger is better. It is the standard
/// indicator for comparing Pareto front approximations because it rewards both convergence and
/// spread. The value is only comparable between runs/libraries when the same reference point is
/// used, so the reference point is always supplied by the caller rather than inferred.
///
/// - Maximized objectives are handled by mirroring them (and the reference) into minimization space.
/// - Scores that do not strictly dominate the reference point contribute nothing and are ignored.
/// - Dominated scores and duplicates are removed before the calculation.
/// - Returns `0.0` if there are no scores, the reference is empty, or a `Multi` objective's
///   dimensions do not match the reference.
///
/// The result is exact. Two objectives use an `O(n log n)` sweep, three objectives slice along
/// the last objective in `O(n²)`, and four or more objectives recursively slice (HSO), whose
/// cost grows exponentially with the number of objectives.
pub fn hypervolume<T: AsRef<[f32]>>(scores: &[T], reference: &[f32], objective: &Objective) -> f32 {
    let dims = reference.len();
    if scores.is_empty() || dims == 0 {
        return 0.0;
    }

    let directions = match objective {
        Objective::Single(opt) => vec![*opt; dims],
        Objective::Multi(opts) if opts.len() == dims => opts.clone(),
        Objective::Multi(_) => return 0.0,
    };

    let to_min_space = |values: &[f32]| {
        values
            .iter()
            .zip(directions.iter())
            .map(|(&v, opt)| match opt {
                Optimize::Minimize => v as f64,
                Optimize::Maximize => -(v as f64),
            })
            .collect::<Vec<f64>>()
    };

    let reference = to_min_space(reference);

    let mut points = scores
        .iter()
        .map(|score| score.as_ref())
        .filter(|score| score.len() == dims)
        .map(to_min_space)
        .filter(|point| point.iter().zip(reference.iter()).all(|(p, r)| p < r))
        .collect::<Vec<Vec<f64>>>();

    points.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    points.dedup();

    let minimize = Objective::Multi(vec![Optimize::Minimize; dims]);
    let front = points
        .iter()
        .filter(|point| {
            !points
                .iter()
                .any(|other| dominance(other, *point, &minimize))
        })
        .map(|point| point.as_slice())
        .collect::<Vec<&[f64]>>();

    sliced_hypervolume(front, &reference, dims) as f32
}

/// Hypervolume of `points` over their first `dims` coordinates (all in minimization space).
/// Slices along the last coordinate: between consecutive values of that coordinate, the
/// dominated cross-section is the `dims - 1` hypervolume of every point already passed.
/// Dominated points are allowed here - they simply add nothing to a cross-section.
fn sliced_hypervolume(mut points: Vec<&[f64]>, reference: &[f64], dims: usize) -> f64 {
    if points.is_empty() {
        return 0.0;
    }

    match dims {
        1 => {
            let best = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
            (reference[0] - best).max(0.0)
        }
        2 => {
            points.sort_unstable_by(|a, b| {
                a[0].partial_cmp(&b[0])
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal))
            });
            sorted_hypervolume_2d(&points, reference)
        }
        _ => {
            let last = dims - 1;
            points.sort_unstable_by(|a, b| {
                a[last]
                    .partial_cmp(&b[last])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut volume = 0.0;
            let mut section: Vec<&[f64]> = Vec::with_capacity(points.len());

            for (i, &point) in points.iter().enumerate() {
                if dims == 3 {
                    // Keep the 2D cross-section sorted so each slice is a linear sweep.
                    let at = section.partition_point(|p| {
                        p[0] < point[0] || (p[0] == point[0] && p[1] <= point[1])
                    });
                    section.insert(at, point);
                } else {
                    section.push(point);
                }

                let top = points.get(i + 1).map_or(reference[last], |next| next[last]);
                let height = top - point[last];

                if height > 0.0 {
                    let area = if dims == 3 {
                        sorted_hypervolume_2d(&section, reference)
                    } else {
                        sliced_hypervolume(section.clone(), reference, last)
                    };

                    volume += height * area;
                }
            }

            volume
        }
    }
}

/// 2D hypervolume of points sorted ascending by their first coordinate (ties by the second).
/// Walks the staircase, adding the rectangle each non-dominated step adds below the last one.
#[inline]
fn sorted_hypervolume_2d(points: &[&[f64]], reference: &[f64]) -> f64 {
    let mut area = 0.0;
    let mut floor = reference[1];

    for point in points {
        if point[1] < floor {
            area += (reference[0] - point[0]) * (floor - point[1]);
            floor = point[1];
        }
    }

    area
}

/// Das-Dennis reference directions on the simplex.
/// Returns Vec of length H = C(p+m-1, m-1), each dir length m and sums to 1.0.
pub fn das_dennis(m: usize, p: usize) -> Vec<Vec<f32>> {
    let mut out = Vec::new();
    let mut current = vec![0usize; m];

    fn rec(
        i: usize,
        m: usize,
        remaining: usize,
        p: usize,
        current: &mut [usize],
        out: &mut Vec<Vec<f32>>,
    ) {
        if i == m - 1 {
            current[i] = remaining;
            let dir = current.iter().map(|&x| x as f32 / p as f32).collect();
            out.push(dir);
            return;
        }
        for x in 0..=remaining {
            current[i] = x;
            rec(i + 1, m, remaining - x, p, current, out);
        }
    }

    rec(0, m, p, p, &mut current, &mut out);
    out
}

pub fn entropy<T, K>(scores: &[T], bins_per_dim: usize) -> K
where
    K: Float + PartialOrd,
    T: AsRef<[K]>,
{
    let n = scores.len();
    let m = if n > 0 { scores[0].as_ref().len() } else { 0 };

    if n == 0 || m == 0 || bins_per_dim == 0 {
        return K::zero();
    }

    let mut mins = vec![K::infinity(); m];
    let mut maxs = vec![K::neg_infinity(); m];

    for row in scores.iter().take(n) {
        let row = row.as_ref();
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
        if (maxs[d] - mins[d]).abs() < K::EPS {
            maxs[d] = mins[d] + K::one();
        }
    }

    let mut cell_counts: HashMap<Vec<u8>, usize> = HashMap::new();

    for row in scores.iter().take(n) {
        let row = row.as_ref();
        let mut cell = Vec::with_capacity(m);

        for d in 0..m {
            let norm = (row[d] - mins[d]) / (maxs[d] - mins[d]);
            let mut idx = (norm * K::from(bins_per_dim).unwrap())
                .floor()
                .to_i32()
                .unwrap();
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

    let n_f = K::from(n).unwrap();
    let mut h = K::zero();
    for &count in cell_counts.values() {
        let p = K::from(count).unwrap() / n_f;
        if p > K::zero() {
            h = h - p * p.ln();
        }
    }

    let k = cell_counts.len().min(n);
    if k > 1 {
        h / K::from(k).unwrap().ln()
    } else {
        K::zero()
    }
}

fn ensure_buffer_len<T: AsRef<[f32]>>(scores: &[T], buffer: &mut [f32]) -> bool {
    let n = scores.len();
    if n == 0 {
        return false;
    }

    let m = scores[0].as_ref().len();
    if m == 0 {
        buffer.fill(0.0);
        return false;
    }

    true
}

#[cfg(test)]
mod tests {

    use super::*;

    fn obj_min2() -> Objective {
        Objective::Multi(vec![Optimize::Minimize, Optimize::Minimize])
    }

    fn obj_max2() -> Objective {
        Objective::Multi(vec![Optimize::Maximize, Optimize::Maximize])
    }

    // ---- crowding_distance ----

    #[test]
    fn crowding_distance_empty() {
        let scores: Vec<Vec<f32>> = vec![];
        let d = crowding_distance(&scores);
        assert!(d.is_empty());
    }

    #[test]
    fn crowding_distance_zero_dims() {
        let scores = vec![vec![], vec![]];
        let d = crowding_distance(&scores);
        assert_eq!(d, vec![0.0, 0.0]);
    }

    #[test]
    fn crowding_distance_two_points_are_infinite() {
        let scores = vec![vec![0.0f32, 0.0], vec![1.0, 1.0]];
        let d = crowding_distance(&scores);
        assert!(d[0].is_infinite());
        assert!(d[1].is_infinite());
    }

    #[test]
    fn crowding_distance_known_values_1d() {
        // 1D case: interior points should get normalized neighbor span.
        // scores: 0, 1, 2, 3  => range = 3
        // interior at 1: (2-0)/3 = 2/3
        // interior at 2: (3-1)/3 = 2/3
        let scores = vec![vec![0.0f32], vec![1.0], vec![2.0], vec![3.0]];
        let d = crowding_distance(&scores);

        assert!(d[0].is_infinite());
        assert!(d[3].is_infinite());

        // allow tiny float error
        assert!((d[1] - (2.0 / 3.0)).abs() < EPSILON, "d[1] = {}", d[1]);
        assert!((d[2] - (2.0 / 3.0)).abs() < EPSILON, "d[2] = {}", d[2]);
    }

    #[test]
    fn crowding_distance_invariant_under_affine_transform_per_dim() {
        // Because each dim is normalized by (max - min), scaling + shifting a dim should not change
        // crowding distances.
        let a = vec![vec![0.0f32], vec![2.0], vec![4.0], vec![7.0], vec![10.0]];
        let b = a.iter().map(|v| vec![v[0] * 3.0 + 5.0]).collect::<Vec<_>>();

        let da = crowding_distance(&a);
        let db = crowding_distance(&b);

        assert_eq!(da.len(), db.len());
        for i in 0..da.len() {
            if da[i].is_infinite() {
                assert!(db[i].is_infinite());
            } else {
                assert!(
                    (da[i] - db[i]).abs() < 1e-6,
                    "i={}: {} vs {}",
                    i,
                    da[i],
                    db[i]
                );
            }
        }
    }

    #[test]
    fn crowding_distance_constant_dim_contributes_nothing() {
        // Second dimension is constant -> should not affect distances (range == 0 -> skipped).
        let scores = vec![
            vec![0.0f32, 5.0],
            vec![1.0, 5.0],
            vec![2.0, 5.0],
            vec![3.0, 5.0],
        ];
        let d = crowding_distance(&scores);

        assert!(d[0].is_infinite());
        assert!(d[3].is_infinite());

        assert!((d[1] - (2.0 / 3.0)).abs() < EPSILON);
        assert!((d[2] - (2.0 / 3.0)).abs() < EPSILON);
    }

    // ---- dominance ----

    #[test]
    fn dominance_minimization_basic() {
        let a = vec![1.0f32, 2.0];
        let b = vec![2.0f32, 3.0];
        assert!(dominance(&a, &b, &obj_min2()));
        assert!(!dominance(&b, &a, &obj_min2()));
    }

    #[test]
    fn dominance_maximization_basic() {
        let a = vec![5.0f32, 5.0];
        let b = vec![4.0f32, 5.0];

        // maximize: a dominates b (>= in all, > in at least one)
        assert!(dominance(&a, &b, &obj_max2()));
        assert!(!dominance(&b, &a, &obj_max2()));
    }

    #[test]
    fn dominance_equal_scores_is_false() {
        let a = vec![1.0f32, 2.0];
        let b = vec![1.0f32, 2.0];
        assert!(!dominance(&a, &b, &obj_min2()));
        assert!(!dominance(&b, &a, &obj_min2()));
    }

    #[test]
    fn dominance_tradeoff_neither_dominates() {
        let a = vec![1.0f32, 10.0];
        let b = vec![2.0f32, 9.0];
        assert!(!dominance(&a, &b, &obj_min2()));
        assert!(!dominance(&b, &a, &obj_min2()));
    }

    // ---- pareto_front / non_dominated ----

    #[test]
    fn non_dominated_matches_pareto_front_indices() {
        let scores = vec![
            vec![1.0f32, 1.0], // ND
            vec![2.0f32, 2.0], // dominated by [1,1]
            vec![1.0f32, 3.0], // tradeoff with [3,1], ND vs many
            vec![3.0f32, 1.0], // tradeoff with [1,3], ND
            vec![4.0f32, 4.0], // dominated
        ];

        let nd_idx = non_dominated(&scores, &obj_min2());
        let front = pareto_front(&scores, &obj_min2());

        let mut nd_vals = nd_idx
            .iter()
            .map(|&i| scores[i].clone())
            .collect::<Vec<_>>();
        nd_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut front_sorted = front.clone();
        front_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        assert_eq!(nd_vals, front_sorted);
    }

    #[test]
    fn pareto_front_contains_all_points_when_no_dominance() {
        // All points trade off -> all non-dominated
        let scores = vec![
            vec![0.0f32, 10.0],
            vec![1.0f32, 9.0],
            vec![2.0f32, 8.0],
            vec![3.0f32, 7.0],
        ];
        let front = pareto_front(&scores, &obj_min2());

        assert_eq!(front.len(), scores.len());
    }

    // ---- rank ----

    #[test]
    fn rank_two_points_dominated_order() {
        // For minimization, [0,0] should be in front 0; [1,1] in front 1.
        let scores = vec![vec![0.0f32, 0.0], vec![1.0f32, 1.0]];
        let r = rank(&scores, &obj_min2());

        assert_eq!(r.len(), 2);
        assert_eq!(r[0], 0, "best point should be rank 0");
        assert_eq!(r[1], 1, "dominated point should be rank 1");
    }

    #[test]
    fn rank_front_partition_expected() {
        // Construct 3 fronts for minimization:
        // F0: A=[0,0]
        // F1: B=[1,0], C=[0,1] (both dominated by A, but neither dominates the other)
        // F2: D=[2,2] (dominated by B and C)
        let a = vec![0.0f32, 0.0];
        let b = vec![1.0f32, 0.0];
        let c = vec![0.0f32, 1.0];
        let d = vec![2.0f32, 2.0];

        let scores = vec![a, b, c, d];
        let r = rank(&scores, &obj_min2());

        assert_eq!(r[0], 0, "A should be in front 0");
        assert_eq!(r[1], 1, "B should be in front 1");
        assert_eq!(r[2], 1, "C should be in front 1");
        assert_eq!(r[3], 2, "D should be in front 2");
    }

    // ---- weights ----

    #[test]
    fn weights_are_positive_and_finite_and_length_matches() {
        let scores = vec![
            vec![0.0f32, 0.0],
            vec![1.0f32, 0.0],
            vec![0.0f32, 1.0],
            vec![2.0f32, 2.0],
        ];

        let w = weights(&scores, &obj_min2());
        assert_eq!(w.len(), scores.len());
        for (i, x) in w.iter().enumerate() {
            assert!(*x > 0.0, "w[{}] should be > 0, got {}", i, x);
            assert!(x.is_finite(), "w[{}] should be finite, got {}", i, x);
        }
    }

    #[test]
    fn weights_prefer_better_front_over_dominated_point() {
        let scores = vec![
            vec![0.0f32, 0.0], // nondominated
            vec![1.0f32, 1.0], // dominated
        ];

        let w = weights(&scores, &obj_min2());
        assert!(w[0] > w[1], "nondominated point should get higher weight");
    }

    #[test]
    fn rank_empty_is_empty() {
        let scores: Vec<Vec<f32>> = vec![];
        let r = rank(&scores, &obj_min2());
        assert!(r.is_empty());
    }

    #[test]
    fn rank_single_is_front0() {
        let scores = vec![vec![1.0f32, 2.0]];
        let r = rank(&scores, &obj_min2());
        assert_eq!(r, vec![0]);
    }

    #[test]
    fn rank_duplicate_points_same_front() {
        // Two identical best points should both be rank 0; dominated point should be later.
        let scores = vec![
            vec![0.0f32, 0.0],
            vec![0.0f32, 0.0], // duplicate
            vec![1.0f32, 1.0], // dominated by both
        ];
        let r = rank(&scores, &obj_min2());

        assert_eq!(r[0], 0);
        assert_eq!(r[1], 0);
        assert_eq!(r[2], 1);
    }

    #[test]
    fn rank_all_nondominated_all_front0() {
        // Tradeoff curve: none dominates another under minimization
        let scores = vec![
            vec![0.0f32, 10.0],
            vec![1.0f32, 9.0],
            vec![2.0f32, 8.0],
            vec![3.0f32, 7.0],
            vec![4.0f32, 6.0],
        ];
        let r = rank(&scores, &obj_min2());
        assert!(
            r.iter().all(|&x| x == 0),
            "expected all rank 0, got {:?}",
            r
        );
    }

    #[test]
    fn rank_strict_chain_increasing_fronts() {
        // Strict dominance chain for minimization:
        // [0,0] dominates [1,1] dominates [2,2] dominates [3,3]
        let scores = vec![
            vec![0.0f32, 0.0],
            vec![1.0f32, 1.0],
            vec![2.0f32, 2.0],
            vec![3.0f32, 3.0],
        ];
        let r = rank(&scores, &obj_min2());
        assert_eq!(r, vec![0, 1, 2, 3]);
    }

    #[test]
    fn rank_matches_iterative_non_dominated_peel() {
        // Property-style test:
        // If we repeatedly peel off the non-dominated set, the peel number should equal rank.
        fn peel_ranks(scores: &[Vec<f32>], objective: &Objective) -> Vec<usize> {
            let mut remaining: Vec<usize> = (0..scores.len()).collect();
            let mut out = vec![usize::MAX; scores.len()];
            let mut front = 0usize;

            while !remaining.is_empty() {
                let subset = remaining.iter().map(|&i| &scores[i]).collect::<Vec<_>>();
                let nd_local = non_dominated(&subset, objective); // indices into subset

                for &k in &nd_local {
                    let global_i = remaining[k];
                    out[global_i] = front;
                }

                // remove the ND points from remaining (in descending order of local indices)
                let mut nd_local_sorted = nd_local;
                nd_local_sorted.sort_unstable_by(|a, b| b.cmp(a));
                for k in nd_local_sorted {
                    remaining.remove(k);
                }

                front += 1;
            }

            out
        }

        let scores = vec![
            vec![0.0f32, 0.0], // F0
            vec![0.0f32, 1.0], // F1
            vec![1.0f32, 0.0], // F1
            vec![1.0f32, 1.0], // F2
            vec![2.0f32, 2.0], // F3
            vec![0.5f32, 0.5], // F2 (dominated by [0,0], not by [0,1] or [1,0])
        ];

        let r = rank(&scores, &obj_min2());
        let p = peel_ranks(&scores, &obj_min2());

        assert_eq!(
            r, p,
            "rank() should match iterative peel ranks\nrank={:?}\npeel={:?}",
            r, p
        );
    }

    // ---- hypervolume ----

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "expected {expected}, got {actual}"
        );
    }

    fn random_points(n: usize, dims: usize) -> Vec<Vec<f32>> {
        use crate::domain::random_provider;
        (0..n)
            .map(|_| {
                (0..dims)
                    .map(|_| random_provider::random::<f32>())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn hypervolume_empty_is_zero() {
        let scores: Vec<Vec<f32>> = vec![];
        assert_eq!(hypervolume(&scores, &[1.0, 1.0], &obj_min2()), 0.0);
    }

    #[test]
    fn hypervolume_single_point_is_its_box() {
        let scores = vec![vec![0.5f32, 0.5]];
        assert_close(hypervolume(&scores, &[1.0, 1.0], &obj_min2()), 0.25);
    }

    #[test]
    fn hypervolume_2d_staircase() {
        // Union of boxes up to (4,4): 3x1 + 2x1 + 1x1 strips = 6
        let scores = vec![vec![1.0f32, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
        assert_close(hypervolume(&scores, &[4.0, 4.0], &obj_min2()), 6.0);
    }

    #[test]
    fn hypervolume_ignores_dominated_duplicate_and_out_of_reference_points() {
        let front = vec![vec![1.0f32, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
        let mut noisy = front.clone();
        noisy.push(vec![2.5, 2.5]); // dominated
        noisy.push(vec![2.0, 2.0]); // duplicate
        noisy.push(vec![0.5, 5.0]); // outside the reference in f2
        noisy.push(vec![4.0, 0.0]); // on the reference boundary in f1 -> zero volume

        assert_close(
            hypervolume(&noisy, &[4.0, 4.0], &obj_min2()),
            hypervolume(&front, &[4.0, 4.0], &obj_min2()),
        );
    }

    #[test]
    fn hypervolume_3d_known_values() {
        let obj = Objective::Multi(vec![Optimize::Minimize; 3]);

        let single = vec![vec![0.0f32, 0.0, 0.0]];
        assert_close(hypervolume(&single, &[1.0, 2.0, 3.0], &obj), 6.0);

        // boxes of volume 4 and 2 overlapping in a unit cube -> 4 + 2 - 1 = 5
        let pair = vec![vec![0.0f32, 0.0, 1.0], vec![1.0, 1.0, 0.0]];
        assert_close(hypervolume(&pair, &[2.0, 2.0, 2.0], &obj), 5.0);
    }

    #[test]
    fn hypervolume_4d_known_values() {
        let obj = Objective::Multi(vec![Optimize::Minimize; 4]);

        // boxes of volume 8 and 2 overlapping in a unit hypercube -> 8 + 2 - 1 = 9
        let pair = vec![vec![0.0f32, 0.0, 0.0, 1.0], vec![1.0, 1.0, 1.0, 0.0]];
        assert_close(hypervolume(&pair, &[2.0; 4], &obj), 9.0);
    }

    #[test]
    fn hypervolume_maximize_mirrors_minimize() {
        let scores = vec![vec![1.0f32, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
        let negated = scores
            .iter()
            .map(|s| s.iter().map(|v| -v).collect::<Vec<f32>>())
            .collect::<Vec<_>>();

        assert_close(
            hypervolume(&negated, &[-4.0, -4.0], &obj_max2()),
            hypervolume(&scores, &[4.0, 4.0], &obj_min2()),
        );
    }

    #[test]
    fn hypervolume_mixed_objectives() {
        // minimize f1, maximize f2 with reference (4, 0): (1,3) box 3x3, (2,4) box 2x4, overlap 2x3
        let obj = Objective::Multi(vec![Optimize::Minimize, Optimize::Maximize]);
        let scores = vec![vec![1.0f32, 3.0], vec![2.0, 4.0]];
        assert_close(hypervolume(&scores, &[4.0, 0.0], &obj), 9.0 + 8.0 - 6.0);
    }

    #[test]
    fn hypervolume_dimension_mismatch_is_zero() {
        let scores = vec![vec![0.5f32, 0.5]];
        let obj = Objective::Multi(vec![Optimize::Minimize; 3]);
        assert_eq!(hypervolume(&scores, &[1.0, 1.0], &obj), 0.0);
    }

    #[test]
    fn hypervolume_higher_dims_agree_with_lower_dim_paths() {
        // Embedding a point set into one more dimension with a constant 0 coordinate and a
        // reference of 1 in that dimension must not change the volume. This checks the 3D
        // slicing path against the 2D sweep and the general (HSO) path against the 3D one.
        crate::domain::random_provider::seed(17);

        for dims in [2usize, 3] {
            let obj = Objective::Multi(vec![Optimize::Minimize; dims]);
            let obj_up = Objective::Multi(vec![Optimize::Minimize; dims + 1]);

            let points = random_points(60, dims);
            let lifted = points
                .iter()
                .map(|p| p.iter().copied().chain([0.0]).collect::<Vec<f32>>())
                .collect::<Vec<_>>();

            let reference = vec![1.1f32; dims];
            let reference_up = vec![1.1f32; dims]
                .into_iter()
                .chain([1.0])
                .collect::<Vec<_>>();

            let hv = hypervolume(&points, &reference, &obj);
            let hv_up = hypervolume(&lifted, &reference_up, &obj_up);

            assert!(hv > 0.0);
            assert!(
                (hv - hv_up).abs() < 1e-4,
                "dims {dims}: {hv} vs lifted {hv_up}"
            );
        }
    }
}
