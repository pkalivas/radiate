use crate::{PyEngineInput, PyEngineInputType};
use radiate::*;

const TOURNAMENT_SELECTOR: &str = "TournamentSelector";
const ROULETTE_WHEEL_SELECTOR: &str = "RouletteWheelSelector";
const RANK_SELECTOR: &str = "RankSelector";
const STEADY_STATE_SELECTOR: &str = "SteadyStateSelector";
const STOCHASTIC_UNIVERSAL_SELECTOR: &str = "StochasticUniversalSamplingSelector";
const BOLTZMANN_SELECTOR: &str = "BoltzmannSelector";
const ELITE_SELECTOR: &str = "EliteSelector";
const RANDOM_SELECTOR: &str = "RandomSelector";
const NSGA2_SELECTOR: &str = "NSGA2Selector";
const TOURNAMENT_NSGA2_SELECTOR: &str = "TournamentNSGA2Selector";

pub trait InputConverter<C: Chromosome, O> {
    fn convert(&self) -> O;
}

impl<C> InputConverter<C, Box<dyn Select<C>>> for PyEngineInput
where
    C: Chromosome + Clone,
{
    fn convert(&self) -> Box<dyn Select<C>> {
        if !matches!(
            self.input_type,
            PyEngineInputType::SurvivorSelector | PyEngineInputType::OffspringSelector
        ) {
            panic!("Input type {:?} not a selector", self.input_type);
        }

        match self.component.as_str() {
            TOURNAMENT_SELECTOR => {
                let tournament_size = self
                    .args
                    .get("tournament_size")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(3);
                Box::new(TournamentSelector::new(tournament_size))
            }
            ROULETTE_WHEEL_SELECTOR => Box::new(RouletteSelector::new()),
            RANK_SELECTOR => Box::new(RankSelector::new()),
            STEADY_STATE_SELECTOR => {
                let steady_state_size = self
                    .args
                    .get("replacement_count")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                Box::new(SteadyStateSelector::new(steady_state_size))
            }
            STOCHASTIC_UNIVERSAL_SELECTOR => Box::new(StochasticUniversalSamplingSelector::new()),
            BOLTZMANN_SELECTOR => {
                let temp = self
                    .args
                    .get("temp")
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(1.0);

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
