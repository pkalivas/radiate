use crate::PyEngineParam;
use radiate::{
    BoltzmannSelector, Chromosome, EliteSelector, GeneticEngineBuilder, LinearRankSelector,
    NSGA2Selector, RankSelector, RouletteSelector, StochasticUniversalSamplingSelector,
    TournamentSelector,
};

pub(crate) fn set_selector<C, T>(
    builder: GeneticEngineBuilder<C, T>,
    selector: &PyEngineParam,
    is_offspring: bool,
) -> GeneticEngineBuilder<C, T>
where
    C: Chromosome,
    T: Clone + Send + Sync,
{
    if selector.name() == "tournament" {
        let args = selector.get_args();
        let tournament_size = args
            .get("k")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        return match is_offspring {
            true => builder.offspring_selector(TournamentSelector::new(tournament_size)),
            false => builder.survivor_selector(TournamentSelector::new(tournament_size)),
        };
    } else if selector.name() == "roulette" {
        return match is_offspring {
            true => builder.offspring_selector(RouletteSelector::new()),
            false => builder.survivor_selector(RouletteSelector::new()),
        };
    } else if selector.name() == "rank" {
        return match is_offspring {
            true => builder.offspring_selector(RankSelector::new()),
            false => builder.survivor_selector(RankSelector::new()),
        };
    } else if selector.name() == "elitism" {
        return match is_offspring {
            true => builder.offspring_selector(EliteSelector::new()),
            false => builder.survivor_selector(EliteSelector::new()),
        };
    } else if selector.name() == "boltzmann" {
        let args = selector.get_args();
        let temperature = args
            .get("temp")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        return match is_offspring {
            true => builder.offspring_selector(BoltzmannSelector::new(temperature)),
            false => builder.survivor_selector(BoltzmannSelector::new(temperature)),
        };
    } else if selector.name() == "stocastic_universal_sampling" {
        return match is_offspring {
            true => builder.offspring_selector(StochasticUniversalSamplingSelector::new()),
            false => builder.survivor_selector(StochasticUniversalSamplingSelector::new()),
        };
    } else if selector.name() == "linear_rank" {
        let args = selector.get_args();
        let selection_pressure = args
            .get("pressure")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        return match is_offspring {
            true => builder.offspring_selector(LinearRankSelector::new(selection_pressure)),
            false => builder.survivor_selector(LinearRankSelector::new(selection_pressure)),
        };
    } else if selector.name() == "nsga2" {
        return match is_offspring {
            true => builder.offspring_selector(NSGA2Selector::new()),
            false => builder.survivor_selector(NSGA2Selector::new()),
        };
    }

    builder
}
