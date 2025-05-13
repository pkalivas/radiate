use crate::PyEngineParam;
use radiate::{
    BoltzmannSelector, Chromosome, EliteSelector, GeneticEngineBuilder, LinearRankSelector,
    NSGA2Selector, RankSelector, RouletteSelector, StochasticUniversalSamplingSelector,
    TournamentSelector,
};

const TOURNAMENT_SELECTOR: &str = "tournament";
const ROULETTE_SELECTOR: &str = "roulette";
const RANK_SELECTOR: &str = "rank";
const ELITISM_SELECTOR: &str = "elitism";
const BOLTZMANN_SELECTOR: &str = "boltzmann";
const STOCHASTIC_UNIVERSAL_SAMPLING_SELECTOR: &str = "stochastic_universal_sampling";
const LINEAR_RANK_SELECTOR: &str = "linear_rank";
const NSGA2_SELECTOR: &str = "nsga2";

pub(crate) fn set_selector<C, T>(
    builder: GeneticEngineBuilder<C, T>,
    selector: &PyEngineParam,
    is_offspring: bool,
) -> GeneticEngineBuilder<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync,
{
    if selector.name() == TOURNAMENT_SELECTOR {
        let args = selector.get_args();
        let tournament_size = args
            .get("k")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        return match is_offspring {
            true => builder.offspring_selector(TournamentSelector::new(tournament_size)),
            false => builder.survivor_selector(TournamentSelector::new(tournament_size)),
        };
    } else if selector.name() == ROULETTE_SELECTOR {
        return match is_offspring {
            true => builder.offspring_selector(RouletteSelector::new()),
            false => builder.survivor_selector(RouletteSelector::new()),
        };
    } else if selector.name() == RANK_SELECTOR {
        return match is_offspring {
            true => builder.offspring_selector(RankSelector::new()),
            false => builder.survivor_selector(RankSelector::new()),
        };
    } else if selector.name() == ELITISM_SELECTOR {
        return match is_offspring {
            true => builder.offspring_selector(EliteSelector::new()),
            false => builder.survivor_selector(EliteSelector::new()),
        };
    } else if selector.name() == BOLTZMANN_SELECTOR {
        let args = selector.get_args();
        let temperature = args
            .get("temp")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        return match is_offspring {
            true => builder.offspring_selector(BoltzmannSelector::new(temperature)),
            false => builder.survivor_selector(BoltzmannSelector::new(temperature)),
        };
    } else if selector.name() == STOCHASTIC_UNIVERSAL_SAMPLING_SELECTOR {
        return match is_offspring {
            true => builder.offspring_selector(StochasticUniversalSamplingSelector::new()),
            false => builder.survivor_selector(StochasticUniversalSamplingSelector::new()),
        };
    } else if selector.name() == LINEAR_RANK_SELECTOR {
        let args = selector.get_args();
        let selection_pressure = args
            .get("pressure")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        return match is_offspring {
            true => builder.offspring_selector(LinearRankSelector::new(selection_pressure)),
            false => builder.survivor_selector(LinearRankSelector::new(selection_pressure)),
        };
    } else if selector.name() == NSGA2_SELECTOR {
        return match is_offspring {
            true => builder.offspring_selector(NSGA2Selector::new()),
            false => builder.survivor_selector(NSGA2Selector::new()),
        };
    }

    builder
}
