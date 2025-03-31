#[derive(Debug, Clone, PartialEq)]
pub struct P2Quantile {
    quantile: f32,               // Target quantile (0 < quantile < 1).
    count: usize,                // Number of observations seen so far.
    positions: [usize; 5],       // Marker positions: n₁, n₂, n₃, n₄, n₅.
    heights: [f32; 5],           // Marker heights (estimates): q₁, q₂, q₃, q₄, q₅.
    increments: [f32; 5],        // Increments for desired positions (dn').
    desired_positions: [f32; 5], // Desired marker positions: n'₁, n'₂, n'₃, n'₄, n'₅.
}

impl P2Quantile {
    /// Create a new P² quantile estimator for target quantile `quantile` (0 < quantile < 1).
    pub fn new(quantile: f32) -> Self {
        let mut p2 = P2Quantile {
            quantile,
            count: 0,
            positions: [0; 5],
            heights: [0.0; 5],
            increments: [0.0; 5],
            desired_positions: [0.0; 5],
        };
        p2.initialize_state();
        p2.set_quantile(quantile);
        p2.finalize_quantiles();
        p2
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.initialize_state();
        self.set_quantile(self.quantile);
        self.finalize_quantiles();
    }

    /// Initialize the internal state.
    fn initialize_state(&mut self) {
        self.count = 0;
        self.positions = [0; 5];
        self.heights = [0.0; 5];
        self.increments = [0.0; 5];
        self.desired_positions = [0.0; 5];
        // The first two increments are fixed.
        self.increments[0] = 0.0;
        self.increments[1] = 1.0;
        // The remaining three will be set by set_quantile.
    }

    /// An insertion sort for small arrays.
    fn smallsort<T: PartialOrd + Copy>(array: &mut [T]) {
        let n = array.len();
        for i in 0..n {
            let mut j = i;
            while j > 0 && array[j] < array[j - 1] {
                array.swap(j, j - 1);
                j -= 1;
            }
        }
    }

    /// Parabolic prediction for marker adjustment.
    fn parabolic(&self, i: usize, d: i32) -> f32 {
        let d = d as f32;
        let n_i = self.positions[i] as f32;
        let n_im1 = self.positions[i - 1] as f32;
        let n_ip1 = self.positions[i + 1] as f32;
        let q_i = self.heights[i];
        let q_im1 = self.heights[i - 1];
        let q_ip1 = self.heights[i + 1];
        let numerator = (n_i - n_im1 + d) * (q_ip1 - q_i) / (n_ip1 - n_i)
            + (n_ip1 - n_i - d) * (q_i - q_im1) / (n_i - n_im1);
        q_i + (d / (n_ip1 - n_im1)) * numerator
    }

    /// Linear prediction as a fallback.
    fn linear(&self, i: usize, d: i32) -> f32 {
        let d = d as f32;
        let n_i = self.positions[i] as f32;
        // When d is negative, i+d is i-1.
        let n_id = self.positions[(i as isize + d as isize) as usize] as f32;
        let q_i = self.heights[i];
        let q_id = self.heights[(i as isize + d as isize) as usize];
        q_i + d * (q_id - q_i) / (n_id - n_i)
    }

    /// Helper: return the sign of a number (+1 or -1).
    fn sign(value: f32) -> i32 {
        if value >= 0.0 { 1 } else { -1 }
    }

    /// Prepare algorithm initialization by storing the first nMarkers (5) values.
    fn prepare_algorithm_initialization(&mut self, value: f32) {
        self.heights[self.count] = value;
        self.count += 1;
        if self.count == 5 {
            self.initialize_algorithm();
        }
    }

    /// Once 5 values are collected, sort the heights and initialize marker positions.
    fn initialize_algorithm(&mut self) {
        Self::smallsort(&mut self.heights);
        for i in 0..5 {
            self.positions[i] = i + 1;
        }
    }

    /// Process a new observation once initialization is complete.
    fn run_algorithm(&mut self, value: f32) {
        let n_markers = 5;
        let cell_index: usize;
        // Step B.1: Determine cell index.
        if value < self.heights[0] {
            self.heights[0] = value;
            cell_index = 1;
        } else if value >= self.heights[n_markers - 1] {
            self.heights[n_markers - 1] = value;
            cell_index = n_markers - 1;
        } else {
            let mut idx = 1;
            for i in 1..n_markers {
                if value < self.heights[i] {
                    idx = i;
                    break;
                }
            }
            cell_index = idx;
        }

        // Step B.2: Update positions and desired positions.
        for i in cell_index..n_markers {
            self.positions[i] += 1;
            self.desired_positions[i] += self.increments[i];
        }
        for i in 0..cell_index {
            self.desired_positions[i] += self.increments[i];
        }

        // Step B.3: Adjust markers 2 through n_markers-1.
        for i in 1..(n_markers - 1) {
            let d = self.desired_positions[i] - self.positions[i] as f32;
            if (d >= 1.0 && (self.positions[i + 1] - self.positions[i]) > 1)
                || (d <= -1.0 && (self.positions[i - 1] as i32 - self.positions[i] as i32) < -1)
            {
                let s = Self::sign(d);
                let newq = self.parabolic(i, s);
                if self.heights[i - 1] < newq && newq < self.heights[i + 1] {
                    self.heights[i] = newq;
                } else {
                    self.heights[i] = self.linear(i, s);
                }
                self.positions[i] = ((self.positions[i] as i32) + s) as usize;
            }
        }
    }

    /// Set the target quantile by updating the increments.
    /// For nQuantiles = 1, we set:
    ///   increments[2] = quantile,
    ///   increments[3] = quantile / 2,
    ///   increments[4] = (1 + quantile) / 2.
    fn set_quantile(&mut self, quantile: f32) {
        self.increments[2] = quantile;
        self.increments[3] = quantile / 2.0;
        self.increments[4] = (1.0 + quantile) / 2.0;
    }

    /// Finalize the quantile settings by sorting increments and initializing desired positions.
    fn finalize_quantiles(&mut self) {
        Self::smallsort(&mut self.increments);
        let n_markers = 5;
        for i in 0..n_markers {
            self.desired_positions[i] = (n_markers - 1) as f32 * self.increments[i] + 1.0;
        }
    }

    /// Add a new observation.
    pub fn add(&mut self, value: f32) {
        if self.count < 5 {
            self.prepare_algorithm_initialization(value);
        } else {
            self.run_algorithm(value);
        }
    }

    /// Return the estimated quantile based on the current state.
    /// If there are fewer than 5 values, returns a closest value based on nearest rank.
    pub fn value(&self) -> f32 {
        self.result_with_quantile(self.increments[2])
    }

    /// Return the estimated quantile for a given quantile parameter.
    pub fn result_with_quantile(&self, quantile: f32) -> f32 {
        let n_markers = 5;
        if self.count < n_markers {
            let mut sorted_heights = self.heights[..self.count].to_vec();
            Self::smallsort(&mut sorted_heights);
            let mut closest = 1;
            for i in 2..self.count {
                if ((i as f32) / (self.count as f32) - quantile).abs()
                    < ((closest as f32) / (n_markers as f32) - quantile).abs()
                {
                    closest = i;
                }
            }
            if self.count == 1 {
                return sorted_heights[0]; // Return the minimum if closest is 0.
            }

            if self.count == 0 {
                return f32::NAN; // No values added.
            }

            sorted_heights[closest]
        } else {
            let mut closest = 1;
            for i in 2..(n_markers - 1) {
                if (self.increments[i] - quantile).abs()
                    < (self.increments[closest] - quantile).abs()
                {
                    closest = i;
                }
            }
            self.heights[closest]
        }
    }
}

impl Default for P2Quantile {
    fn default() -> Self {
        P2Quantile::new(0.5) // Default to median.
    }
}

#[cfg(test)]
mod tests {
    use super::P2Quantile;

    #[test]
    fn test_p2_quantile() {
        let mut pq = P2Quantile::new(0.5); // Median
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        for value in values {
            pq.add(value);
        }

        let result = pq.value();
        assert!(result >= 5.0 && result <= 6.0);
    }

    #[test]
    fn test_p2_quantile_with_custom_quantile() {
        let mut pq = P2Quantile::new(0.75);
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        for value in values {
            pq.add(value);
        }

        let result = pq.result_with_quantile(0.75);
        assert!(result >= 7.0 && result <= 8.0);
    }

    #[test]
    fn test_p2_quantile_empty() {
        let pq = P2Quantile::new(0.5);
        let result = pq.value();
        assert!(result.is_nan());
    }

    #[test]
    fn test_p2_quantile_single_value() {
        let mut pq = P2Quantile::new(0.5);
        pq.add(5.0);

        let result = pq.value();
        assert_eq!(result, 5.0);
    }
}
