use radiate_core::{Chromosome, Objective, Optimize, Population, Select, pareto};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

const EPS: f32 = 1e-12;

#[derive(Debug, Clone)]
pub struct NSGA3Selector {
    ref_dirs: Arc<Mutex<Vec<Vec<f32>>>>,
    partitions: usize,
}

impl NSGA3Selector {
    pub fn new(partitions: usize) -> Self {
        Self {
            ref_dirs: Arc::new(Mutex::new(Vec::new())),
            partitions,
        }
    }

    fn reference_dirs(&self, dims: usize) -> Vec<Vec<f32>> {
        let mut dirs = self.ref_dirs.lock().unwrap();

        if dirs.is_empty() {
            *dirs = pareto::das_dennis(dims, self.partitions);
        }

        dirs.clone()
    }
}

impl<C: Chromosome + Clone> Select<C> for NSGA3Selector {
    fn name(&self) -> &'static str {
        "nsga3_selector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        if population.is_empty() || count == 0 {
            return Population::empty();
        }

        let scores = population
            .iter_scores()
            .map(|score| to_minimization_space(score.as_ref(), objective))
            .collect::<Vec<_>>();

        let min_objective = minimization_objective(objective.dims());
        let ranks = pareto::rank(&scores, &min_objective);
        let fronts = fronts_from_ranks(&ranks);

        let ref_dirs = self.reference_dirs(objective.dims());

        let mut selected = Vec::with_capacity(count);
        let mut front_idx = 0usize;

        while front_idx < fronts.len() && selected.len() + fronts[front_idx].len() <= count {
            selected.extend_from_slice(&fronts[front_idx]);
            front_idx += 1;
        }

        if selected.len() < count && front_idx < fronts.len() {
            let remaining = count - selected.len();

            selected.extend(niching_fill(
                &scores,
                &ref_dirs,
                &selected,
                &fronts[front_idx],
                remaining,
            ));
        }

        selected
            .into_iter()
            .take(count)
            .map(|idx| population[idx].clone())
            .collect()
    }
}

#[inline]
fn minimization_objective(dims: usize) -> Objective {
    Objective::Multi(vec![Optimize::Minimize; dims])
}

#[inline]
pub fn to_minimization_space(score: &[f32], objective: &Objective) -> Vec<f32> {
    match objective {
        Objective::Single(opt) => {
            if *opt == Optimize::Minimize {
                score.to_vec()
            } else {
                score.iter().map(|&x| -x).collect()
            }
        }
        Objective::Multi(opts) => score
            .iter()
            .zip(opts.iter())
            .map(|(&x, opt)| if *opt == Optimize::Minimize { x } else { -x })
            .collect(),
    }
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

#[derive(Debug, Clone)]
pub struct ObjectiveBounds {
    ideal: Vec<f32>,
    nadir: Vec<f32>,
}

impl ObjectiveBounds {
    pub fn from_scores(scores: &[Vec<f32>]) -> Self {
        if scores.is_empty() {
            return Self {
                ideal: Vec::new(),
                nadir: Vec::new(),
            };
        }

        let dims = scores[0].len();
        let mut ideal = vec![f32::INFINITY; dims];
        let mut nadir = vec![f32::NEG_INFINITY; dims];

        for score in scores {
            for dim in 0..dims {
                ideal[dim] = ideal[dim].min(score[dim]);
                nadir[dim] = nadir[dim].max(score[dim]);
            }
        }

        Self { ideal, nadir }
    }

    pub fn normalize(&self, score: &[f32]) -> Vec<f32> {
        score
            .iter()
            .enumerate()
            .map(|(dim, &value)| {
                let den = self.nadir[dim] - self.ideal[dim];

                if !den.is_finite() || den.abs() <= EPS {
                    0.0
                } else {
                    (value - self.ideal[dim]) / den
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Association {
    idx: usize,
    niche: usize,
    distance: f32,
}

/// Given:
/// - `already_selected`: indices chosen from earlier fronts
/// - `last_front`: indices in the partial front
///
/// Returns additional indices from `last_front` using NSGA-III niching.
pub fn niching_fill(
    scores: &[Vec<f32>],
    ref_dirs: &[Vec<f32>],
    already_selected: &[usize],
    last_front: &[usize],
    remaining: usize,
) -> Vec<usize> {
    if remaining == 0 || last_front.is_empty() || ref_dirs.is_empty() {
        return Vec::new();
    }

    let bounds = ObjectiveBounds::from_scores(scores);
    let mut niche_count = vec![0usize; ref_dirs.len()];

    for &idx in already_selected {
        let normalized = bounds.normalize(&scores[idx]);
        let (niche, _) = nearest_reference_direction(&normalized, ref_dirs);
        niche_count[niche] += 1;
    }

    let mut candidates = last_front
        .iter()
        .map(|&idx| {
            let normalized = bounds.normalize(&scores[idx]);
            let (niche, distance) = nearest_reference_direction(&normalized, ref_dirs);

            Association {
                idx,
                niche,
                distance,
            }
        })
        .collect::<Vec<_>>();

    let mut picked = Vec::with_capacity(remaining);

    while picked.len() < remaining && !candidates.is_empty() {
        let niche = least_crowded_candidate_niche(&candidates, &niche_count);
        let candidate_idx = closest_candidate_in_niche(&candidates, niche);

        let selected = candidates.swap_remove(candidate_idx);

        picked.push(selected.idx);
        niche_count[selected.niche] += 1;
    }

    picked
}

#[inline]
fn least_crowded_candidate_niche(candidates: &[Association], niche_count: &[usize]) -> usize {
    candidates
        .iter()
        .map(|candidate| candidate.niche)
        .min_by_key(|&niche| niche_count[niche])
        .unwrap()
}

#[inline]
fn closest_candidate_in_niche(candidates: &[Association], niche: usize) -> usize {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.niche == niche)
        .min_by(|(_, a), (_, b)| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        })
        .map(|(idx, _)| idx)
        .unwrap()
}

#[inline]
pub fn nearest_reference_direction(point: &[f32], ref_dirs: &[Vec<f32>]) -> (usize, f32) {
    let mut best = (0usize, f32::INFINITY);

    for (idx, direction) in ref_dirs.iter().enumerate() {
        let direction_norm = dot(direction, direction);

        if direction_norm <= EPS || !direction_norm.is_finite() {
            continue;
        }

        let projection = dot(point, direction) / direction_norm;
        let distance = perpendicular_distance(point, direction, projection);

        if distance < best.1 {
            best = (idx, distance);
        }
    }

    best
}

#[inline]
fn perpendicular_distance(point: &[f32], direction: &[f32], projection: f32) -> f32 {
    point
        .iter()
        .zip(direction)
        .map(|(&p, &d)| {
            let diff = p - projection * d;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

#[inline]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(&x, &y)| x * y).sum()
}

// use radiate_core::{Chromosome, Objective, Optimize, Population, Select, pareto};
// use std::sync::{Arc, Mutex};

// #[derive(Debug, Clone)]
// pub struct NSGA3Selector {
//     ref_dirs: Arc<Mutex<Vec<Vec<f32>>>>,
//     num_refs: usize,
// }

// impl NSGA3Selector {
//     pub fn new(ref_points: usize) -> Self {
//         Self {
//             ref_dirs: Arc::new(Mutex::new(Vec::new())),
//             num_refs: ref_points,
//         }
//     }

//     fn create_ref_dirs_if_needed(&self, num_objectives: usize, ref_points: usize) {
//         let mut dirs = self.ref_dirs.lock().unwrap();
//         if dirs.is_empty() {
//             *dirs = pareto::das_dennis(num_objectives, ref_points);
//         }
//     }
// }

// impl<C: Chromosome + Clone> Select<C> for NSGA3Selector {
//     fn name(&self) -> &'static str {
//         "nsga3_selector"
//     }

//     fn select(
//         &self,
//         population: &Population<C>,
//         objective: &Objective,
//         count: usize,
//     ) -> Population<C> {
//         self.create_ref_dirs_if_needed(objective.dims(), self.num_refs);
//         let raw = population.iter_scores().collect::<Vec<_>>();

//         let scores_min = raw
//             .iter()
//             .map(|s| to_minimization_space(s.as_ref(), objective))
//             .collect::<Vec<_>>();

//         let ranks = pareto::rank(&scores_min, objective);

//         let fronts = fronts_from_ranks(&ranks);

//         let mut selected: Vec<usize> = Vec::with_capacity(count);
//         let mut fi = 0usize;

//         while fi < fronts.len() && selected.len() + fronts[fi].len() <= count {
//             selected.extend_from_slice(&fronts[fi]);
//             fi += 1;
//         }

//         if selected.len() < count && fi < fronts.len() {
//             let remaining = count - selected.len();
//             let extra = nsga3_niching_fill(
//                 &scores_min,
//                 &self.ref_dirs.lock().unwrap(),
//                 &selected,
//                 &fronts[fi],
//                 remaining,
//             );
//             selected.extend(extra);
//         }

//         selected
//             .into_iter()
//             .take(count)
//             .map(|i| population[i].clone())
//             .collect::<Population<C>>()
//     }
// }

// #[inline]
// pub fn fronts_from_ranks(ranks: &[usize]) -> Vec<Vec<usize>> {
//     if ranks.is_empty() {
//         return Vec::new();
//     }
//     let max_rank = *ranks.iter().max().unwrap_or(&0);
//     let mut fronts = vec![Vec::<usize>::new(); max_rank + 1];
//     for (i, &r) in ranks.iter().enumerate() {
//         fronts[r].push(i);
//     }

//     while fronts.last().is_some_and(|front| front.is_empty()) {
//         fronts.pop();
//     }

//     fronts
// }

// #[inline]
// pub fn to_minimization_space(score: &[f32], objective: &Objective) -> Vec<f32> {
//     match objective {
//         Objective::Single(opt) => {
//             if *opt == Optimize::Minimize {
//                 score.to_vec()
//             } else {
//                 score.iter().map(|&x| -x).collect()
//             }
//         }
//         Objective::Multi(opts) => score
//             .iter()
//             .zip(opts.iter())
//             .map(|(&x, opt)| if *opt == Optimize::Minimize { x } else { -x })
//             .collect(),
//     }
// }

// #[inline]
// pub fn min_max_points(scores_min: &[Vec<f32>]) -> (Vec<f32>, Vec<f32>) {
//     let n = scores_min.len();
//     if n == 0 {
//         return (Vec::new(), Vec::new());
//     }
//     let m = scores_min[0].len();
//     let mut ideal = vec![f32::INFINITY; m];
//     let mut nadir = vec![f32::NEG_INFINITY; m];

//     for s in scores_min {
//         for d in 0..m {
//             ideal[d] = ideal[d].min(s[d]);
//             nadir[d] = nadir[d].max(s[d]);
//         }
//     }
//     (ideal, nadir)
// }

// #[inline]
// pub fn normalize_minmax(s: &[f32], ideal: &[f32], nadir: &[f32]) -> Vec<f32> {
//     let m = s.len();
//     let mut out = vec![0.0f32; m];
//     for d in 0..m {
//         let den = (nadir[d] - ideal[d]).abs();
//         if !den.is_finite() || den <= 1e-12 {
//             out[d] = 0.0;
//         } else {
//             out[d] = (s[d] - ideal[d]) / den;
//         }
//     }
//     out
// }

// #[inline]
// fn dot(a: &[f32], b: &[f32]) -> f32 {
//     a.iter().zip(b).map(|(&x, &y)| x * y).sum()
// }

// #[inline]
// fn norm2(a: &[f32]) -> f32 {
//     dot(a, a)
// }

// /// Returns (k, distance) where k is reference dir index.
// #[inline]
// pub fn associate_with_dist(y: &[f32], ref_dirs: &[Vec<f32>]) -> (usize, f32) {
//     let mut best_k = 0usize;
//     let mut best_d = f32::INFINITY;

//     for (k, r) in ref_dirs.iter().enumerate() {
//         let rr = norm2(r);
//         if rr <= 1e-12 || !rr.is_finite() {
//             continue;
//         }
//         let t = dot(y, r) / rr; // projection scalar
//         // dist^2 = ||y - t r||^2
//         let mut d2 = 0.0f32;
//         for i in 0..y.len() {
//             let diff = y[i] - t * r[i];
//             d2 += diff * diff;
//         }
//         if d2 < best_d {
//             best_d = d2;
//             best_k = k;
//         }
//     }

//     (best_k, best_d.sqrt())
// }

// /// Given:
// /// - already_selected: indices already chosen (from earlier fronts)
// /// - last_front: indices in the partial front
// ///   returns additional indices from last_front to reach `remaining`.
// pub fn nsga3_niching_fill(
//     scores: &[Vec<f32>], // minimization-space scores
//     ref_dirs: &[Vec<f32>],
//     already_selected: &[usize],
//     last_front: &[usize],
//     remaining: usize,
// ) -> Vec<usize> {
//     if remaining == 0 || last_front.is_empty() {
//         return Vec::new();
//     }

//     // normalize all needed points (simple min/max)
//     let (ideal, nadir) = min_max_points(scores);

//     let mut niche_count = vec![0usize; ref_dirs.len()];

//     // count niches from already-selected
//     for &idx in already_selected {
//         let y = normalize_minmax(&scores[idx], &ideal, &nadir);
//         let (k, _) = associate_with_dist(&y, ref_dirs);
//         niche_count[k] += 1;
//     }

//     // candidates in last front: (idx, niche, dist)
//     let mut cand: Vec<(usize, usize, f32)> = last_front
//         .iter()
//         .map(|&idx| {
//             let y = normalize_minmax(&scores[idx], &ideal, &nadir);
//             let (k, d) = associate_with_dist(&y, ref_dirs);
//             (idx, k, d)
//         })
//         .collect();

//     let mut picked = Vec::with_capacity(remaining);

//     while picked.len() < remaining && !cand.is_empty() {
//         // choose niche with smallest niche_count among those that still have candidates
//         let mut best_k = None;
//         let mut best_cnt = usize::MAX;

//         for &(_, k, _) in &cand {
//             let c = niche_count[k];
//             if c < best_cnt {
//                 best_cnt = c;
//                 best_k = Some(k);
//             }
//         }
//         let k = best_k.unwrap();

//         // pick candidate in niche k:
//         // common rule: if niche_count[k]==0 pick smallest distance; else also smallest distance (simple + works well)
//         let mut best_i = None;
//         let mut best_d = f32::INFINITY;

//         for (i, &(_, kk, d)) in cand.iter().enumerate() {
//             if kk == k && d < best_d {
//                 best_d = d;
//                 best_i = Some(i);
//             }
//         }

//         let (idx, _, _) = cand.swap_remove(best_i.unwrap());
//         picked.push(idx);
//         niche_count[k] += 1;
//     }

//     picked
// }
