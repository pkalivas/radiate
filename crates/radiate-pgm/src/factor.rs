/// Potential function associated with a factor in a probabilistic graphical model.
#[derive(Clone, Debug, PartialEq)]
pub enum Potential {
    /// Log-table for discrete variables (factor over discrete parents/children)
    DiscreteTable(Vec<f32>),
    /// Unary Gaussian: p(x) = N(x | μ, σ^2)
    /// (Mean, Variance)
    Gaussian(f32, f32),
    /// Linear-Gaussian: p(y | x_vec) = N(y | w·x + b, σ^2)  (child must be Real)
    LinearGaussian {
        weights: Vec<f64>,
        bias: f64,
        var: f64,
    },
    /// Mixture of Gaussians selected by discrete parents (optional, good for hybrid BN)
    MixtureByDiscrete {
        // For each parent assignment (flattened row-major), a Gaussian component
        mean: Vec<f64>,
        var: Vec<f64>,
    },
}

/// A factor specification in a probabilistic graphical model.
/// The factor defines a potential function over a set of variables.
#[derive(Clone, Debug, PartialEq)]
pub struct FactorSpec {
    pub potential: Potential, // must match domains of scope
}
