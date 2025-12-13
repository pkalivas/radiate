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
    pub fn new(metric: Arc<dyn Distance<P>>) -> Self {
        CentroidClusterer {
            centroids: Vec::new(),
            metric,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EuclideanDistance;
    use std::sync::Arc;

    #[test]
    fn test_centroid_clusterer() {
        let metric = EuclideanDistance;
        let mut clusterer = CentroidClusterer::new(Arc::new(metric));

        let points = vec![
            vec![1.0, 2.0],
            vec![1.5, 1.8],
            vec![5.0, 8.0],
            vec![8.0, 8.0],
            vec![1.0, 0.6],
            vec![9.0, 11.0],
        ];

        let threshold = 3.0;
        let mut assignments = Vec::new();
        for point in &points {
            let (idx, dist) = clusterer.assign_or_create(point, Some(threshold));
            assignments.push((idx, dist));
        }

        assert_eq!(clusterer.len(), 3);
        assert_eq!(assignments[0].0, assignments[1].0); // first two points in same cluster
        assert_eq!(assignments[0].0, assignments[4].0); // first and fifth points in same cluster
        assert_ne!(assignments[0].0, assignments[2].0); // third point in different cluster
        assert_eq!(assignments[2].0, assignments[3].0); // fourth point in different cluster
    }
}
