use crate::diversity::Distance;
use crate::math::knn::KNN;
use std::sync::Arc;

/// Simple online clustering over a set of centroids.
///
/// - `P`: point type (e.g. Vec<f32>, Genotype<C>, etc.)
pub struct CentroidClusterer<P> {
    centroids: Vec<P>,
    metric: Arc<dyn Distance<P>>,
}

impl<P> CentroidClusterer<P> {
    pub fn new(metric: impl Into<Arc<dyn Distance<P>>>) -> Self {
        CentroidClusterer {
            centroids: Vec::new(),
            metric: metric.into(),
        }
    }

    pub fn with_centroids(mut self, centroids: Vec<P>) -> Self {
        self.centroids = centroids;
        self
    }

    /// Returns a slice of current centroids.
    pub fn centroids(&self) -> &[P] {
        &self.centroids
    }

    /// Returns the number of centroids.
    pub fn len(&self) -> usize {
        self.centroids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.centroids.is_empty()
    }
}

impl<P: Clone> CentroidClusterer<P> {
    /// Assign a point to the nearest centroid if within `threshold`.
    /// Otherwise, create a new centroid with this point as its center.
    ///
    /// Returns the index of the assigned/created centroid and the distance.
    pub fn assign_or_create(&mut self, point: &P, threshold: Option<f32>) -> (usize, f32) {
        if self.centroids.is_empty() {
            self.centroids.push(point.clone());
            return (0, 0.0);
        }

        let mut knn = KNN::new(&self.centroids, Arc::clone(&self.metric));
        let result = knn.query_point(point, 1);

        if let Some(&(idx, dist)) = result.cluster.first() {
            if let Some(threshold) = threshold {
                if dist <= threshold {
                    (idx, dist)
                } else {
                    let new_idx = self.centroids.len();
                    self.centroids.push(point.clone());
                    (new_idx, dist)
                }
            } else {
                (idx, dist)
            }
        } else {
            let new_idx = self.centroids.len();
            self.centroids.push(point.clone());
            (new_idx, 0.0)
        }
    }
}
