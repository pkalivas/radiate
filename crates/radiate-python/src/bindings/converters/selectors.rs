use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::*;

impl<C> InputTransform<RadiateResult<Box<dyn Select<C>>>> for PyEngineInput
where
    C: Chromosome + Clone,
{
    fn transform(&self) -> RadiateResult<Box<dyn Select<C>>> {
        if !matches!(
            self.input_type,
            PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector
        ) {
            return Err(radiate_err!(Builder: format!(
                "Expected input type to be SurvivorSelector or OffspringSelector, got {:?}",
                self.input_type
            )));
        }

        match self.component.as_str() {
            crate::constants::TOURNAMENT_SELECTOR => {
                let tournament_size = self.extract::<i64>("k")?;
                Ok(Box::new(TournamentSelector::new(tournament_size as usize)))
            }
            crate::constants::ROULETTE_WHEEL_SELECTOR => Ok(Box::new(RouletteSelector::new())),
            crate::constants::RANK_SELECTOR => Ok(Box::new(RankSelector::new())),
            crate::constants::STOCHASTIC_UNIVERSAL_SELECTOR => {
                Ok(Box::new(StochasticUniversalSamplingSelector::new()))
            }
            crate::constants::BOLTZMANN_SELECTOR => {
                let temp = self.extract::<f64>("temp")?;
                Ok(Box::new(BoltzmannSelector::new(temp as f32)))
            }
            crate::constants::ELITE_SELECTOR => Ok(Box::new(EliteSelector::new())),
            crate::constants::RANDOM_SELECTOR => Ok(Box::new(RandomSelector::new())),
            crate::constants::NSGA2_SELECTOR => Ok(Box::new(NSGA2Selector::new())),
            crate::constants::NSGA3_SELECTOR => {
                let ref_points = self.extract::<i64>("points")?;
                Ok(Box::new(NSGA3Selector::new(ref_points as usize)))
            }
            crate::constants::TOURNAMENT_NSGA2_SELECTOR => {
                Ok(Box::new(TournamentNSGA2Selector::new()))
            }
            crate::constants::LINEAR_RANK_SELECTOR => {
                let selection_pressure = self.extract::<f64>("pressure")?;
                Ok(Box::new(LinearRankSelector::new(selection_pressure as f32)))
            }
            _ => Err(radiate_err!(Builder: format!(
                "Selector type {} not yet implemented",
                self.component
            ))),
        }
    }
}
