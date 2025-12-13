use crate::diversity::Distance;
use std::{cmp::Ordering, sync::Arc};

pub struct KnnQueryResult<'a> {
    pub cluster: &'a [(usize, f32)],
    pub max_distance: f32,
    pub min_distance: f32,
}

impl<'a> KnnQueryResult<'a> {
    pub fn new(cluster: &'a [(usize, f32)], max_distance: f32, min_distance: f32) -> Self {
        KnnQueryResult {
            cluster,
            max_distance,
            min_distance,
        }
    }

    pub fn average_distance(&self) -> f32 {
        if self.cluster.is_empty() {
            0.0
        } else {
            let total = self.cluster.iter().map(|(_, dist)| dist).sum::<f32>();
            total / (self.cluster.len() as f32)
        }
    }
}

/// Brute-force KNN index over a slice of points.
///
/// - `P`: point type, must implement `KnnPoint`
/// - `M`: distance metric
pub struct KNN<'a, P> {
    points: &'a [P],
    metric: Arc<dyn Distance<P>>,
    scratch: Vec<(usize, f32)>,
}

impl<'a, P> KNN<'a, P> {
    #[inline]
    pub fn new(points: &'a [P], metric: impl Into<Arc<dyn Distance<P>>>) -> Self {
        let len = points.len();
        KNN {
            points,
            metric: metric.into(),
            scratch: Vec::with_capacity(len.saturating_sub(1)),
        }
    }

    /// Returns a reference to the underlying points.
    #[inline]
    pub fn points(&self) -> &'a [P] {
        self.points
    }

    /// Query the k nearest neighbors of the point at `query_index`.
    ///
    /// - `k`: number of neighbors to return
    /// - `exclude_self`: if true, will skip the point at `query_index`
    ///
    /// Returns a slice of `(index, distance)` sorted by increasing distance.
    /// The slice is backed by an internal scratch buffer and is invalidated
    /// by the next query.
    pub fn query_index(&mut self, query_index: usize, k: usize) -> KnnQueryResult<'_> {
        let len = self.points.len();
        if len == 0 || k == 0 {
            self.scratch.clear();
            return KnnQueryResult::new(&self.scratch, f32::NEG_INFINITY, f32::INFINITY);
        }

        let points = &self.points[query_index];
        self.query_point_internal(points, Some(query_index), k)
    }

    /// Query the k nearest neighbors of an arbitrary query point (not
    /// necessarily in the index).
    pub fn query_point(&mut self, query: &P, k: usize) -> KnnQueryResult<'_> {
        if self.points.is_empty() || k == 0 {
            self.scratch.clear();
            return KnnQueryResult::new(&self.scratch, f32::NEG_INFINITY, f32::INFINITY);
        }

        self.query_point_internal(query, None, k)
    }

    #[inline]
    fn query_point_internal(
        &mut self,
        query: &P,
        query_index: Option<usize>,
        k: usize,
    ) -> KnnQueryResult<'_> {
        self.scratch.clear();

        let mut min_distance = f32::INFINITY;
        let mut max_distance = f32::NEG_INFINITY;
        for (idx, p) in self.points.iter().enumerate() {
            if let Some(qi) = query_index {
                if qi == idx {
                    continue;
                }
            }

            let dist = self.metric.distance(query, p).max(1e-12);
            min_distance = min_distance.min(dist);
            max_distance = max_distance.max(dist);
            self.scratch.push((idx, dist));
        }

        let n = self.scratch.len();
        if n == 0 || k == 0 {
            self.scratch.clear();
            return KnnQueryResult::new(&self.scratch, max_distance, min_distance);
        }

        if k >= n {
            self.scratch
                .sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
            return KnnQueryResult::new(&self.scratch, max_distance, min_distance);
        }

        let (left, _, _) = self.scratch.select_nth_unstable_by(k - 1, |a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal)
        });

        left.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

        self.scratch.truncate(k);

        KnnQueryResult::new(&self.scratch, max_distance, min_distance)
    }
}
