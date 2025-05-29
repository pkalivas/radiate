use crate::PyEngineParam;
use radiate::{
    BoltzmannSelector, Chromosome, EliteSelector, Epoch, GeneticEngineBuilder, LinearRankSelector,
    NSGA2Selector, RankSelector, RouletteSelector, StochasticUniversalSamplingSelector,
    TournamentSelector,
};

use super::ParamMapper;

const TOURNAMENT_SELECTOR: &str = "tournament";
const ROULETTE_SELECTOR: &str = "roulette";
const RANK_SELECTOR: &str = "rank";
const ELITISM_SELECTOR: &str = "elitism";
const BOLTZMANN_SELECTOR: &str = "boltzmann";
const STOCHASTIC_UNIVERSAL_SAMPLING_SELECTOR: &str = "stochastic_universal_sampling";
const LINEAR_RANK_SELECTOR: &str = "linear_rank";
const NSGA2_SELECTOR: &str = "nsga2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectorType {
    Tournament,
    Roulette,
    Rank,
    Elitism,
    Boltzmann,
    StochasticUniversalSampling,
    LinearRank,
    NSGA2,
}

pub enum SelectorConfig {
    Tournament(TournamentSelectorMapper),
    Roulette(RouletteSelectorMapper),
    Rank(RankSelectorMapper),
    Elitism(EliteSelectorMapper),
    Boltzmann(BoltzmannSelectorMapper),
    StochasticUniversalSampling(StochasticUniversalSamplingSelectorMapper),
    LinearRank(LinearRankSelectorMapper),
    NSGA2(NSGA2SelectorMapper),
}

pub(super) fn selector_name_to_type(name: &str) -> SelectorType {
    match name {
        TOURNAMENT_SELECTOR => SelectorType::Tournament,
        ROULETTE_SELECTOR => SelectorType::Roulette,
        RANK_SELECTOR => SelectorType::Rank,
        ELITISM_SELECTOR => SelectorType::Elitism,
        BOLTZMANN_SELECTOR => SelectorType::Boltzmann,
        STOCHASTIC_UNIVERSAL_SAMPLING_SELECTOR => SelectorType::StochasticUniversalSampling,
        LINEAR_RANK_SELECTOR => SelectorType::LinearRank,
        NSGA2_SELECTOR => SelectorType::NSGA2,
        _ => panic!("Unknown selector type"),
    }
}

pub struct TournamentSelectorMapper;

impl<C: Chromosome> ParamMapper<C> for TournamentSelectorMapper {
    type Output = TournamentSelector;

    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let args = param.get_args();
        let tournament_size = args
            .get("k")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        TournamentSelector::new(tournament_size)
    }
}

pub struct RouletteSelectorMapper;
impl<C: Chromosome> ParamMapper<C> for RouletteSelectorMapper {
    type Output = RouletteSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        RouletteSelector::new()
    }
}

pub struct RankSelectorMapper;
impl<C: Chromosome> ParamMapper<C> for RankSelectorMapper {
    type Output = RankSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        RankSelector::new()
    }
}

pub struct EliteSelectorMapper;
impl<C: Chromosome> ParamMapper<C> for EliteSelectorMapper {
    type Output = EliteSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        EliteSelector::new()
    }
}

pub struct BoltzmannSelectorMapper;
impl<C: Chromosome> ParamMapper<C> for BoltzmannSelectorMapper {
    type Output = BoltzmannSelector;

    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let args = param.get_args();
        let temperature = args
            .get("temp")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        BoltzmannSelector::new(temperature)
    }
}

pub struct StochasticUniversalSamplingSelectorMapper;
impl<C: Chromosome> ParamMapper<C> for StochasticUniversalSamplingSelectorMapper {
    type Output = StochasticUniversalSamplingSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        StochasticUniversalSamplingSelector::new()
    }
}

pub struct LinearRankSelectorMapper;
impl<C: Chromosome> ParamMapper<C> for LinearRankSelectorMapper {
    type Output = LinearRankSelector;

    fn map(&self, param: &PyEngineParam) -> Self::Output {
        let args = param.get_args();
        let selection_pressure = args
            .get("pressure")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        LinearRankSelector::new(selection_pressure)
    }
}

pub struct NSGA2SelectorMapper;
impl<C: Chromosome> ParamMapper<C> for NSGA2SelectorMapper {
    type Output = NSGA2Selector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        NSGA2Selector::new()
    }
}

pub(crate) fn set_selector<C, T, E>(
    builder: GeneticEngineBuilder<C, T, E>,
    selector: &PyEngineParam,
    is_offspring: bool,
) -> GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send + Sync,
    E: Epoch<Chromosome = C>,
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
