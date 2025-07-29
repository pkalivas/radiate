use crate::{InputTransform, PyEngineInput, PyEngineInputType};
use radiate::*;

const TOURNAMENT_SELECTOR: &str = "TournamentSelector";
const ROULETTE_WHEEL_SELECTOR: &str = "RouletteSelector";
const RANK_SELECTOR: &str = "RankSelector";
const STEADY_STATE_SELECTOR: &str = "SteadyStateSelector";
const STOCHASTIC_UNIVERSAL_SELECTOR: &str = "StochasticUniversalSamplingSelector";
const BOLTZMANN_SELECTOR: &str = "BoltzmannSelector";
const ELITE_SELECTOR: &str = "EliteSelector";
const RANDOM_SELECTOR: &str = "RandomSelector";
const NSGA2_SELECTOR: &str = "NSGA2Selector";
const TOURNAMENT_NSGA2_SELECTOR: &str = "TournamentNSGA2Selector";

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
            TOURNAMENT_SELECTOR => {
                let tournament_size = self.get_usize("tournament_size").unwrap_or(3);
                Box::new(TournamentSelector::new(tournament_size))
            }
            ROULETTE_WHEEL_SELECTOR => Box::new(RouletteSelector::new()),
            RANK_SELECTOR => Box::new(RankSelector::new()),
            STEADY_STATE_SELECTOR => {
                let steady_state_size = self.get_usize("replacement_count").unwrap_or(1);
                Box::new(SteadyStateSelector::new(steady_state_size))
            }
            STOCHASTIC_UNIVERSAL_SELECTOR => Box::new(StochasticUniversalSamplingSelector::new()),
            BOLTZMANN_SELECTOR => {
                let temp = self.get_f32("temp").unwrap_or(1.0);
                Box::new(BoltzmannSelector::new(temp))
            }
            ELITE_SELECTOR => Box::new(EliteSelector::new()),
            RANDOM_SELECTOR => Box::new(RandomSelector::new()),
            NSGA2_SELECTOR => Box::new(NSGA2Selector::new()),
            TOURNAMENT_NSGA2_SELECTOR => Box::new(TournamentNSGA2Selector::new()),
            _ => {
                panic!("Selector type {} not yet implemented", self.component);
            }
        }
    }
}
