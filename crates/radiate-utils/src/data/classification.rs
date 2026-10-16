use crate::{DataSet, Float, random_provider};
use std::f64::consts::PI;

const IRIS_CSV: &str = include_str!("iris.csv");
const IRIS_CLASSES: [&str; 3] = ["Setosa", "Versicolor", "Virginica"];

/// Inner circle's radius relative to the outer one in [DataSet::circles].
const CIRCLES_FACTOR: f64 = 0.5;

/// How many full turns each arm of [DataSet::spirals] makes.
const SPIRAL_TURNS: f64 = 2.0;

/// Classification datasets. Binary tasks have a single 0/1 label column; multi-class tasks
/// have one one-hot column per class. Generated datasets draw from [random_provider], so
/// they follow `random_provider::seed` and `RADIATE_SEED`, and come out grouped by class:
/// call [DataSet::shuffle] before splitting. [DataSet::iris] comes out already shuffled.
impl<F: Float> DataSet<F> {
    /// The XOR truth table: two binary inputs, label 1 when exactly one is set.
    /// The same as [DataSet::parity] with 2 bits.
    pub fn xor() -> DataSet<F> {
        Self::parity(2)
    }

    /// Every `bits`-wide bit pattern (2^`bits` samples, most significant bit first), labelled 1
    /// when an odd number of bits are set.
    ///
    /// ```text
    ///   parity(3):   0 0 0 → 0    0 1 1 → 0    1 1 0 → 0
    ///                0 0 1 → 1    1 0 0 → 1    1 1 1 → 1
    ///                0 1 0 → 1    1 0 1 → 0
    /// ```
    ///
    /// # Panics
    /// If `bits` is 0 or over 24 (16M samples).
    pub fn parity(bits: usize) -> DataSet<F> {
        assert!(
            (1..=24).contains(&bits),
            "parity needs 1 to 24 bits, got {bits}"
        );

        let (inputs, answers) = (0..1usize << bits)
            .map(|pattern| {
                let input = (0..bits)
                    .rev()
                    .map(|bit| F::from(pattern >> bit & 1).unwrap())
                    .collect();
                (input, vec![F::from(pattern.count_ones() % 2).unwrap()])
            })
            .unzip();

        DataSet::new(inputs, answers)
    }

    /// A basic memory task: input is 1 when the index is a multiple of 4, output is 1 when the index is a multiple of 3.
    pub fn simple_memory(n: usize) -> DataSet<F> {
        let mut inputs = Vec::new();
        let mut answers = Vec::new();

        let input_temp = vec![vec![F::ZERO], vec![F::ZERO], vec![F::ZERO], vec![F::ONE]];
        let answer_temp = vec![vec![F::ZERO], vec![F::ZERO], vec![F::ONE], vec![F::ZERO]];

        for i in 0..n {
            inputs.push(input_temp[i % 4].clone());
            answers.push(answer_temp[i % 4].clone());
        }

        DataSet::new(inputs, answers)
    }

    /// Fisher's iris dataset: 150 samples, 4 features (sepal length, sepal width, petal
    /// length, petal width) and 3 one-hot label columns: Setosa, Versicolor, Virginica.
    /// Rows are shuffled with [random_provider], so the order follows the seed.
    ///
    /// Dataset is shuffled before being returned.
    pub fn iris() -> DataSet<F> {
        let (inputs, answers) = IRIS_CSV
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let mut fields = line.split(',');
                let input = fields
                    .by_ref()
                    .take(4)
                    .map(|value| F::from(value.trim().parse::<f64>().unwrap()).unwrap())
                    .collect();

                let class = fields.next().unwrap().trim().trim_matches('"');
                let index = IRIS_CLASSES.iter().position(|c| *c == class).unwrap();
                let mut label = vec![F::ZERO; IRIS_CLASSES.len()];
                label[index] = F::ONE;

                (input, label)
            })
            .unzip();

        DataSet::new(inputs, answers).shuffle()
    }

    /// Two interleaving half circles. Class 0 is the upper arc, centered on the origin; class
    /// 1 is the lower arc, shifted right and down so it hooks into the first. Gaussian noise
    /// with standard deviation `noise` is added to both coordinates.
    pub fn moons(n: usize, noise: f64) -> DataSet<F> {
        two_classes(n, noise, |class, t| {
            let angle = PI * t;
            if class == 0 {
                (angle.cos(), angle.sin())
            } else {
                (1.0 - angle.cos(), 0.5 - angle.sin())
            }
        })
    }

    /// Two concentric circles. Class 0 is the outer circle (radius 1), class 1 the inner one
    /// (radius 0.5). Gaussian noise with standard deviation `noise` is added to both coordinates.
    pub fn circles(n: usize, noise: f64) -> DataSet<F> {
        two_classes(n, noise, |class, t| {
            let angle = 2.0 * PI * t;
            let radius = if class == 0 { 1.0 } else { CIRCLES_FACTOR };
            (radius * angle.cos(), radius * angle.sin())
        })
    }

    /// Two intertwined spirals, each making two turns out from the origin; class 1 is class 0
    /// rotated half a turn. A classic hard benchmark for NEAT and GP. Gaussian noise with
    /// standard deviation `noise` is added to both coordinates.
    pub fn spirals(n: usize, noise: f64) -> DataSet<F> {
        two_classes(n, noise, |class, t| {
            let angle = 2.0 * PI * SPIRAL_TURNS * t + PI * class as f64;
            (t * angle.cos(), t * angle.sin())
        })
    }
}

/// Builds a two-class, two-feature dataset of `n` samples: `n / 2` of class 0, the rest
/// of class 1. `point(class, t)` places a sample at position `t` in `[0, 1)` along its
/// class's shape; each coordinate then gets gaussian noise.
fn two_classes<F: Float>(
    n: usize,
    noise: f64,
    point: impl Fn(usize, f64) -> (f64, f64),
) -> DataSet<F> {
    let counts = [n / 2, n - n / 2];

    let (inputs, answers) = counts
        .iter()
        .enumerate()
        .flat_map(|(class, &count)| (0..count).map(move |i| (class, i as f64 / count as f64)))
        .map(|(class, t)| {
            let (x, y) = point(class, t);
            let jitter = |v: f64| {
                if noise > 0.0 {
                    v + random_provider::gaussian(0.0, noise)
                } else {
                    v
                }
            };

            let input = vec![F::from(jitter(x)).unwrap(), F::from(jitter(y)).unwrap()];
            (input, vec![F::from(class).unwrap()])
        })
        .unzip();

    DataSet::new(inputs, answers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(data: &DataSet<f32>) -> Vec<f32> {
        data.iter().map(|(_, label)| label[0]).collect()
    }

    #[test]
    fn test_xor_truth_table() {
        let data = DataSet::<f32>::xor();
        let rows = data.iter().collect::<Vec<_>>();

        assert_eq!(data.shape(), (4, 2, 1));
        assert_eq!(rows[0], (&[0.0, 0.0][..], &[0.0][..]));
        assert_eq!(rows[1], (&[0.0, 1.0][..], &[1.0][..]));
        assert_eq!(rows[2], (&[1.0, 0.0][..], &[1.0][..]));
        assert_eq!(rows[3], (&[1.0, 1.0][..], &[0.0][..]));
    }

    #[test]
    fn test_parity_labels_count_set_bits() {
        let data = DataSet::<f32>::parity(5);
        assert_eq!(data.shape(), (32, 5, 1));

        for (input, label) in data.iter() {
            let ones = input.iter().filter(|&&b| b == 1.0).count();
            assert_eq!(label[0], (ones % 2) as f32);
        }
    }

    #[test]
    #[should_panic(expected = "parity needs 1 to 24 bits")]
    fn test_parity_rejects_zero_bits() {
        DataSet::<f32>::parity(0);
    }

    #[test]
    fn test_iris_shape_and_classes() {
        let data = DataSet::<f32>::iris();
        assert_eq!(data.shape(), (150, 4, 3));

        // Exactly 50 per class, every label one-hot.
        let mut per_class = [0; 3];
        for (_, label) in data.iter() {
            assert_eq!(label.iter().sum::<f32>(), 1.0);
            per_class[label.iter().position(|&v| v == 1.0).unwrap()] += 1;
        }
        assert_eq!(per_class, [50, 50, 50]);

        // Shuffled, so look the CSV's first row up instead of expecting it first.
        assert!(
            data.iter()
                .any(|(x, y)| x == [5.1, 3.5, 1.4, 0.2] && y == [1.0, 0.0, 0.0])
        );
    }

    #[test]
    fn test_generated_shapes_and_balance() {
        for data in [
            DataSet::<f32>::moons(101, 0.1),
            DataSet::<f32>::circles(101, 0.1),
            DataSet::<f32>::spirals(101, 0.1),
        ] {
            assert_eq!(data.shape(), (101, 2, 1));

            let labels = labels(&data);
            assert_eq!(labels.iter().filter(|&&l| l == 0.0).count(), 50);
            assert_eq!(labels.iter().filter(|&&l| l == 1.0).count(), 51);
        }
    }

    #[test]
    fn test_generated_follow_the_seed() {
        let make = || {
            random_provider::scoped_seed(7, || {
                (
                    DataSet::<f32>::moons(40, 0.2).features(),
                    DataSet::<f32>::circles(40, 0.2).features(),
                    DataSet::<f32>::spirals(40, 0.2).features(),
                )
            })
        };

        assert_eq!(make(), make());
    }

    #[test]
    fn test_circles_without_noise_lie_on_their_radius() {
        let data = DataSet::<f64>::circles(20, 0.0);
        for (input, label) in data.iter() {
            let radius = (input[0] * input[0] + input[1] * input[1]).sqrt();
            let want = if label[0] == 0.0 { 1.0 } else { CIRCLES_FACTOR };
            assert!(
                (radius - want).abs() < 1e-12,
                "radius {radius}, want {want}"
            );
        }
    }

    #[test]
    fn test_spiral_arms_are_point_reflections() {
        let data = DataSet::<f64>::spirals(40, 0.0);
        let rows = data.features();
        let (arm0, arm1) = rows.split_at(20);

        for (a, b) in arm0.iter().zip(arm1) {
            assert!((a[0] + b[0]).abs() < 1e-12 && (a[1] + b[1]).abs() < 1e-12);
        }
    }
}
