use crate::Float;
use crate::random_provider;
use crate::{DataSet, Primitive};
use std::f64::consts::PI;
use std::ops::Range;

/// Side length of the [DataSet::pagie1] grid: -5.0 to 5.0 in steps of 0.4.
const PAGIE_GRID: usize = 26;

/// Regression datasets. The symbolic regression benchmarks follow their published definitions
/// (McDermott et al., "Genetic Programming Needs Better Benchmarks", 2012), including how
/// points are sampled: those defined on random points draw from [random_provider], so they
/// follow `random_provider::seed` and `RADIATE_SEED`.
impl<F: Float> DataSet<F> {
    pub fn cubic() -> DataSet<F> {
        let mut inputs = Vec::new();
        let mut answers = Vec::new();

        let mut input = -1.0;
        for _ in -10..10 {
            input += 0.1;
            let input_as_f = input.extract::<F>().unwrap();
            let computed = F::FOUR * input_as_f.powf(F::THREE) - F::THREE * input_as_f.powf(F::TWO)
                + input_as_f;

            inputs.push(vec![input_as_f]);
            answers.push(vec![computed]);
        }

        DataSet::new(inputs, answers)
    }

    /// Koza-1: `x⁴ + x³ + x² + x` at 20 random points in `[-1, 1)`.
    pub fn koza1() -> DataSet<F> {
        sample_uniform(20, 1, -1.0..1.0, |x| {
            let x = x[0];
            x.powi(4) + x.powi(3) + x.powi(2) + x
        })
    }

    /// Nguyen-7: `ln(x + 1) + ln(x² + 1)` at 20 random points in `[0, 2)`.
    pub fn nguyen7() -> DataSet<F> {
        sample_uniform(20, 1, 0.0..2.0, |x| {
            let x = x[0];
            (x + 1.0).ln() + (x * x + 1.0).ln()
        })
    }

    /// Pagie-1: `1 / (1 + x⁻⁴) + 1 / (1 + y⁻⁴)` on a 26 × 26 grid over `[-5, 5]` in steps of
    /// 0.4 (676 samples). Deterministic. The grid never hits 0, so `x⁻⁴` stays finite.
    pub fn pagie1() -> DataSet<F> {
        let axis = |i: usize| -5.0 + 0.4 * i as f64;
        let term = |v: f64| 1.0 / (1.0 + v.powi(-4));

        let (inputs, answers) = (0..PAGIE_GRID * PAGIE_GRID)
            .map(|i| {
                let (x, y) = (axis(i / PAGIE_GRID), axis(i % PAGIE_GRID));
                (
                    vec![F::from(x).unwrap(), F::from(y).unwrap()],
                    vec![F::from(term(x) + term(y)).unwrap()],
                )
            })
            .unzip();

        DataSet::new(inputs, answers)
    }

    /// Friedman-1: `10·sin(π·x₁·x₂) + 20·(x₃ − 0.5)² + 10·x₄ + 5·x₅` over 10 inputs drawn from
    /// `[0, 1)`, plus gaussian noise with standard deviation `noise` on the output. Inputs
    /// x₆ to x₁₀ don't affect the output, so a good model learns to ignore them.
    pub fn friedman1(n: usize, noise: f64) -> DataSet<F> {
        sample_uniform(n, 10, 0.0..1.0, |x| {
            let y = 10.0 * (PI * x[0] * x[1]).sin()
                + 20.0 * (x[2] - 0.5).powi(2)
                + 10.0 * x[3]
                + 5.0 * x[4];

            if noise > 0.0 {
                y + random_provider::gaussian(0.0, noise)
            } else {
                y
            }
        })
    }
}

/// `n` samples of `width` inputs, each drawn uniformly from `range`, labelled with `f(inputs)`.
fn sample_uniform<F: Float>(
    n: usize,
    width: usize,
    range: Range<f64>,
    mut f: impl FnMut(&[f64]) -> f64,
) -> DataSet<F> {
    let (inputs, answers) = (0..n)
        .map(|_| {
            let x = (0..width)
                .map(|_| random_provider::range(range.clone()))
                .collect::<Vec<f64>>();
            let y = f(&x);

            (
                x.into_iter().map(|v| F::from(v).unwrap()).collect(),
                vec![F::from(y).unwrap()],
            )
        })
        .unzip();

    DataSet::new(inputs, answers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_koza1_and_nguyen7_follow_their_definitions() {
        let koza = DataSet::<f64>::koza1();
        assert_eq!(koza.shape(), (20, 1, 1));
        for (x, y) in koza.iter() {
            let x = x[0];
            assert!((-1.0..1.0).contains(&x));
            assert_eq!(y[0], x.powi(4) + x.powi(3) + x.powi(2) + x);
        }

        let nguyen = DataSet::<f64>::nguyen7();
        assert_eq!(nguyen.shape(), (20, 1, 1));
        for (x, y) in nguyen.iter() {
            let x = x[0];
            assert!((0.0..2.0).contains(&x));
            assert_eq!(y[0], (x + 1.0).ln() + (x * x + 1.0).ln());
        }
    }

    #[test]
    fn test_pagie1_grid() {
        let data = DataSet::<f64>::pagie1();
        assert_eq!(data.shape(), (676, 2, 1));

        let rows = data.iter().collect::<Vec<_>>();
        assert_eq!(rows[0].0, &[-5.0, -5.0]);
        assert!((rows[675].0[0] - 5.0).abs() < 1e-12 && (rows[675].0[1] - 5.0).abs() < 1e-12);

        // Symmetric in each input, finite everywhere, and both terms in [0, 1).
        let corner = 2.0 / (1.0 + 5f64.powi(-4));
        assert!((rows[0].1[0] - corner).abs() < 1e-12);
        for (x, y) in data.iter() {
            assert!(x.iter().all(|&v| v != 0.0));
            assert!(y[0].is_finite() && (0.0..2.0).contains(&y[0]));
        }
    }

    #[test]
    fn test_friedman1_ignores_trailing_inputs() {
        let data = DataSet::<f64>::friedman1(50, 0.0);
        assert_eq!(data.shape(), (50, 10, 1));

        for (x, y) in data.iter() {
            assert!(x.iter().all(|v| (0.0..1.0).contains(v)));
            let want = 10.0 * (PI * x[0] * x[1]).sin()
                + 20.0 * (x[2] - 0.5).powi(2)
                + 10.0 * x[3]
                + 5.0 * x[4];
            assert_eq!(y[0], want);
        }
    }

    #[test]
    fn test_random_benchmarks_follow_the_seed() {
        let make = || {
            random_provider::scoped_seed(11, || {
                (
                    DataSet::<f32>::koza1().features(),
                    DataSet::<f32>::nguyen7().labels(),
                    DataSet::<f32>::friedman1(30, 0.5).labels(),
                )
            })
        };

        assert_eq!(make(), make());
    }
}
