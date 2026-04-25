use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::*;

impl<C> InputTransform<Box<dyn Select<C>>> for PyEngineInput
where
    C: Chromosome + Clone,
{
    fn transform(&self) -> Box<dyn Select<C>> {
        if !matches!(
            self.input_type,
            PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector
        ) {
            panic!("Input type {:?} not a selector", self.input_type);
        }

        match self.component.as_str() {
            crate::names::TOURNAMENT_SELECTOR => {
                let tournament_size = self.get_usize("tournament_size").unwrap_or(3);
                Box::new(TournamentSelector::new(tournament_size))
            }
            crate::names::ROULETTE_WHEEL_SELECTOR => Box::new(RouletteSelector::new()),
            crate::names::RANK_SELECTOR => Box::new(RankSelector::new()),
            crate::names::STOCHASTIC_UNIVERSAL_SELECTOR => {
                Box::new(StochasticUniversalSamplingSelector::new())
            }
            crate::names::BOLTZMANN_SELECTOR => {
                let temp = self.get_f32("temp").unwrap_or(1.0);
                Box::new(BoltzmannSelector::new(temp))
            }
            crate::names::ELITE_SELECTOR => Box::new(EliteSelector::new()),
            crate::names::RANDOM_SELECTOR => Box::new(RandomSelector::new()),
            crate::names::NSGA2_SELECTOR => Box::new(NSGA2Selector::new()),
            crate::names::NSGA3_SELECTOR => {
                let ref_points = self
                    .get_usize("points")
                    .expect("NSGA3Selector requires 'points' argument");
                Box::new(NSGA3Selector::new(ref_points))
            }
            crate::names::TOURNAMENT_NSGA2_SELECTOR => Box::new(TournamentNSGA2Selector::new()),
            crate::names::LINEAR_RANK_SELECTOR => {
                let selection_pressure = self.get_f32("pressure").unwrap_or(1.0).max(1.0); // Ensure selection pressure is at least 1.0
                Box::new(LinearRankSelector::new(selection_pressure))
            }
            _ => {
                panic!("Selector type {} not yet implemented", self.component);
            }
        }
    }
}
