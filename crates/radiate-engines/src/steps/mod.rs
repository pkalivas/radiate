pub mod evaluate;
pub mod filter;
pub mod front;
pub mod metric;
pub mod recombine;
pub mod speciate;

pub use evaluate::*;
pub use filter::*;
pub use front::*;
pub use metric::*;

use radiate_core::{Chromosome, Ecosystem, MetricSet};
use radiate_error::Result;

use radiate_utils::ToSnakeCase;
pub use recombine::*;
pub use speciate::*;

use crate::EvolutionContext;

pub trait EngineStep<C, T>: Send + Sync
where
    C: Chromosome,
{
    fn name(&self) -> &'static str {
        let name = std::any::type_name::<Self>()
            .split("<")
            .next()
            .unwrap_or(std::any::type_name::<Self>())
            .split("::")
            .last()
            .unwrap_or("Unknown Step");

        let snake_case = name.to_snake_case();

        let mut parts = snake_case
            .split('_')
            .filter(|part| !part.is_empty() && !part.contains("step"))
            .collect::<Vec<_>>();

        parts.insert(0, "step");
        let result = parts.join(".");

        radiate_utils::intern!(result)
    }

    /// The slice of the [EvolutionContext] this step is allowed to touch.
    type View<'a>
    where
        C: 'a,
        T: 'a;

    /// Borrow exactly what this step needs out of the context. This is the
    /// only place a step sees the whole context.
    fn view<'a>(ctx: &'a mut EvolutionContext<C, T>) -> Self::View<'a>;

    fn run_step(&mut self, ctx: &mut EvolutionContext<C, T>) -> Result<()> {
        self.execute(Self::view(ctx))
    }

    fn execute(&mut self, view: Self::View<'_>) -> Result<()>;
}

/// The view most steps need: the generation index, the ecosystem and the metrics.
pub struct EcosystemView<'a, C: Chromosome> {
    pub index: usize,
    pub ecosystem: &'a mut Ecosystem<C>,
    pub metrics: &'a mut MetricSet,
}

impl<'a, C: Chromosome> EcosystemView<'a, C> {
    pub fn new<T>(ctx: &'a mut EvolutionContext<C, T>) -> Self {
        EcosystemView {
            index: ctx.index,
            ecosystem: &mut ctx.ecosystem,
            metrics: &mut ctx.metrics,
        }
    }
}

impl<C, T, F> EngineStep<C, T> for F
where
    C: Chromosome,
    F: FnMut(&mut EvolutionContext<C, T>) -> Result<()> + Send + Sync,
{
    type View<'a>
        = &'a mut EvolutionContext<C, T>
    where
        C: 'a,
        T: 'a;

    fn view<'a>(ctx: &'a mut EvolutionContext<C, T>) -> Self::View<'a> {
        ctx
    }

    fn execute(&mut self, ctx: &mut EvolutionContext<C, T>) -> Result<()> {
        (self)(ctx)
    }
}
