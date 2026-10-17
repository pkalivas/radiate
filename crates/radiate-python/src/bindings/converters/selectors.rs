use crate::{
    PyEngineInput, PyEngineInputType,
    bindings::converters::{FromPyEngineInput, Registry},
    registry,
};
use radiate::*;

pub(crate) fn selectors<C>() -> Registry<Box<dyn Select<C>>>
where
    C: Chromosome + Clone + 'static,
{
    use crate::constants::components;
    registry!("selector", |s| Box::new(s) as Box<dyn Select<C>>; {
        components::TOURNAMENT_SELECTOR           => TournamentSelector,
        components::ROULETTE_WHEEL_SELECTOR       => RouletteSelector,
        components::RANK_SELECTOR                 => RankSelector,
        components::STOCHASTIC_UNIVERSAL_SELECTOR => SUSSelector,
        components::BOLTZMANN_SELECTOR            => BoltzmannSelector,
        components::ELITE_SELECTOR                => EliteSelector,
        components::RANDOM_SELECTOR               => RandomSelector,
        components::NSGA2_SELECTOR                => NSGA2Selector,
        components::NSGA3_SELECTOR                => NSGA3Selector,
        components::TOURNAMENT_NSGA2_SELECTOR     => TournamentNSGA2Selector,
        components::LINEAR_RANK_SELECTOR          => LinearRankSelector,
    })
}

impl<C> FromPyEngineInput for Box<dyn Select<C>>
where
    C: Chromosome + Clone + 'static,
{
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        if !matches!(
            input.input_type,
            PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector
        ) {
            return Err(radiate_err!(Builder: format!(
                "Expected input type to be SurvivorSelector or OffspringSelector, got {:?}",
                input.input_type
            )));
        }

        selectors::<C>().convert(input)
    }
}

/// ---------------------------------------------------------------------------
/// Concrete converters
/// ---------------------------------------------------------------------------
impl FromPyEngineInput for TournamentSelector {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        let k = input.extract::<i64>("k")?;
        Ok(TournamentSelector::new(k as usize))
    }
}

impl FromPyEngineInput for RouletteSelector {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(RouletteSelector::new())
    }
}

impl FromPyEngineInput for RankSelector {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(RankSelector::new())
    }
}

impl FromPyEngineInput for SUSSelector {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(SUSSelector::new())
    }
}

impl FromPyEngineInput for BoltzmannSelector {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        let temp = input.extract::<f64>("temp")?;
        Ok(BoltzmannSelector::new(temp as f32))
    }
}

impl FromPyEngineInput for EliteSelector {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(EliteSelector::new())
    }
}

impl FromPyEngineInput for RandomSelector {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(RandomSelector::new())
    }
}

impl FromPyEngineInput for NSGA2Selector {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(NSGA2Selector::new())
    }
}

impl FromPyEngineInput for NSGA3Selector {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        let points = input.extract::<i64>("points")?;
        Ok(NSGA3Selector::new(points as usize))
    }
}

impl FromPyEngineInput for TournamentNSGA2Selector {
    fn from_input(_: &PyEngineInput) -> RadiateResult<Self> {
        Ok(TournamentNSGA2Selector::new())
    }
}

impl FromPyEngineInput for LinearRankSelector {
    fn from_input(input: &PyEngineInput) -> RadiateResult<Self> {
        let pressure = input.extract::<f64>("pressure")?;
        Ok(LinearRankSelector::new(pressure as f32))
    }
}
