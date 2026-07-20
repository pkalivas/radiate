use crate::{MetricSet, error::RadiateResult};
use radiate_error::radiate_bail;
pub use radiate_expr::*;
use radiate_utils::SmallStr;
use smallvec::SmallVec;

const DEFAULT_VALUE: f32 = 1.0;

#[derive(Clone)]
pub struct RateSet {
    pub control: Expr,
    pub internal: Vec<Expr>,
    pub rate_cache: SmallVec<[f32; 8]>,
    pub last_update_index: usize,
    pub has_updated: bool,
}

impl RateSet {
    pub fn new(control: impl Into<Expr>) -> Self {
        Self {
            control: control.into(),
            internal: Vec::new(),
            rate_cache: SmallVec::new(),
            last_update_index: 0,
            has_updated: false,
        }
    }

    pub fn calculate_rates(
        &mut self,
        generation: usize,
        metrics: &MetricSet,
    ) -> RadiateResult<&[f32]> {
        if generation > self.last_update_index || !self.has_updated {
            self.rate_cache.clear();

            let control_rate = Self::try_eval_rate(metrics, &mut self.control)?;
            self.rate_cache.push(control_rate);

            for expr in &mut self.internal {
                let rate = Self::try_eval_rate(metrics, expr)?;
                self.rate_cache.push(rate);
            }

            self.last_update_index = generation;
            self.has_updated = true;
        }

        Ok(&self.rate_cache)
    }

    pub fn calculate_control_rate(
        &mut self,
        generation: usize,
        metrics: &MetricSet,
    ) -> RadiateResult<f32> {
        self.calculate_rates(generation, metrics)?;
        Ok(self.rate_cache[0])
    }

    pub fn rates(&self) -> &[f32] {
        &self.rate_cache
    }

    pub fn alias(mut self, name: impl Into<SmallStr>) -> Self {
        let name = name.into();
        self.control = self.control.clone().alias(name);
        self
    }

    pub fn push(mut self, expr: impl Into<Expr>) -> Self {
        self.internal.push(expr.into());
        self
    }

    fn try_eval_rate(metrics: &MetricSet, expr: &mut Expr) -> RadiateResult<f32> {
        if let Some(metric) = metrics.get(expr.name()) {
            return Ok(metric.last_value());
        }

        let output = expr.eval(metrics)?;
        match output.extract::<f32>() {
            Some(rate) => Ok(rate),
            None => {
                radiate_bail!(Expr:
                    "Failed to evaluate rate expression for alterer: expected f32 value, got {:?}",
                    output
                );
            }
        }
    }
}

impl Default for RateSet {
    fn default() -> Self {
        Self {
            control: Expr::lit(DEFAULT_VALUE),
            internal: Vec::new(),
            rate_cache: SmallVec::new(),
            last_update_index: 0,
            has_updated: false,
        }
    }
}

impl From<Expr> for RateSet {
    fn from(expr: Expr) -> Self {
        Self::new(expr)
    }
}
