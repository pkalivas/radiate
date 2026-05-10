//! Reusable benchmark problems. Each one is small, fast-converging, and
//! has a known optimum so tests can assert reaching it within a budget.
//!
//! These are deliberately tiny — the goal is to exercise the engine
//! plumbing, not to be a serious benchmark suite. If a future test
//! needs a harder problem, add it here so other tests can share it.

#![allow(dead_code)]

use radiate_core::*;

/// One-Max: maximize the count of `true` bits in a fixed-length bitstring.
/// Optimum is `n_bits`. Trivial to solve,
pub struct OneMax {
    pub n_bits: usize,
    codec: BitCodec<Vec<bool>>,
}

impl OneMax {
    pub fn new(n_bits: usize) -> Self {
        Self {
            n_bits,
            codec: BitCodec::vector(n_bits),
        }
    }

    pub fn optimum(&self) -> i32 {
        self.n_bits as i32
    }
}

impl Problem<BitChromosome, Vec<bool>> for OneMax {
    fn encode(&self) -> Genotype<BitChromosome> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<bool> {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<BitChromosome>) -> Result<Score, RadiateError> {
        let geno = self.decode(individual);
        Ok(geno.iter().filter(|b| **b).count().into())
    }
}

/// Int-minimize-to-zero: minimize the sum of integer chromosome values
/// drawn from `0..max`. Optimum is 0 (all genes at zero).
pub struct IntMinimizeToZero {
    codec: IntCodec<i32, Vec<i32>>,
}

impl IntMinimizeToZero {
    pub fn new(n_genes: usize, max: i32) -> Self {
        Self {
            codec: IntCodec::vector(n_genes, 0..max),
        }
    }
}

impl Problem<IntChromosome<i32>, Vec<i32>> for IntMinimizeToZero {
    fn encode(&self) -> Genotype<IntChromosome<i32>> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<IntChromosome<i32>>) -> Vec<i32> {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<IntChromosome<i32>>) -> Result<Score, RadiateError> {
        let geno = self.decode(individual);
        Ok(geno.iter().sum::<i32>().into())
    }
}

/// Quadratic regression: fit `a*x^2 + b*x + c` to a dense set of `(x, y)`
/// samples drawn from `y = x^2` over `[-2, 2]`. Optimum is MSE → 0
/// (achieved by the coefficients `a=1, b=0, c=0`).
pub struct QuadraticRegression {
    n_samples: usize,
    codec: FloatCodec<f32, Vec<f32>>,
}

impl QuadraticRegression {
    pub fn new(n_samples: usize) -> Self {
        Self {
            n_samples,
            codec: FloatCodec::vector(3, -5.0..5.0),
        }
    }

    /// Returns a closure that computes MSE for a candidate `(a, b, c)`.
    pub fn fitness_fn(&self) -> impl Fn(Vec<f32>) -> f32 + Send + Sync + 'static {
        let n = self.n_samples;
        let test_inputs: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1).max(1) as f32; // [0, 1]
                -2.0 + 4.0 * t // map to [-2, 2]
            })
            .collect();
        let targets: Vec<f32> = test_inputs.iter().map(|x| x * x).collect();

        move |coeffs: Vec<f32>| -> f32 {
            test_inputs
                .iter()
                .zip(targets.iter())
                .map(|(x, y)| {
                    let pred = coeffs[0] * x * x + coeffs[1] * x + coeffs[2];
                    (pred - y).powi(2)
                })
                .sum::<f32>()
                / test_inputs.len() as f32
        }
    }
}

impl Problem<FloatChromosome<f32>, Vec<f32>> for QuadraticRegression {
    fn encode(&self) -> Genotype<FloatChromosome<f32>> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<FloatChromosome<f32>>) -> Vec<f32> {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<FloatChromosome<f32>>) -> Result<Score, RadiateError> {
        let geno = self.decode(individual);
        Ok(self.fitness_fn()(geno).into())
    }
}

/// Sphere function: minimize sum of squares of float genes. Smooth,
/// convex, continuously differentiable.
pub struct Sphere {
    codec: FloatCodec<f32, Vec<f32>>,
}

impl Sphere {
    pub fn new(n_genes: usize, range: f32) -> Self {
        Self {
            codec: FloatCodec::vector(n_genes, -range..range),
        }
    }
}

impl Problem<FloatChromosome<f32>, Vec<f32>> for Sphere {
    fn encode(&self) -> Genotype<FloatChromosome<f32>> {
        self.codec.encode()
    }

    fn decode(&self, genotype: &Genotype<FloatChromosome<f32>>) -> Vec<f32> {
        self.codec.decode(genotype)
    }

    fn eval(&self, individual: &Genotype<FloatChromosome<f32>>) -> Result<Score, RadiateError> {
        let geno = self.decode(individual);
        Ok(geno.iter().map(|x| x * x).sum::<f32>().into())
    }
}
