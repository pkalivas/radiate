use super::{SelectorConfig, SelectorRegistry, SelectorType, mappers::*};

pub fn register_selector_mappers(registry: &mut SelectorRegistry) {
    registry.register_selector(
        SelectorType::Boltzmann,
        SelectorConfig::Boltzmann(BoltzmannSelectorMapper),
    );
    registry.register_selector(
        SelectorType::Tournament,
        SelectorConfig::Tournament(TournamentSelectorMapper),
    );
    registry.register_selector(
        SelectorType::Roulette,
        SelectorConfig::Roulette(RouletteSelectorMapper),
    );
    registry.register_selector(SelectorType::Rank, SelectorConfig::Rank(RankSelectorMapper));
    registry.register_selector(
        SelectorType::Elitism,
        SelectorConfig::Elitism(EliteSelectorMapper),
    );
    registry.register_selector(
        SelectorType::StochasticUniversalSampling,
        SelectorConfig::StochasticUniversalSampling(StochasticUniversalSamplingSelectorMapper),
    );
    registry.register_selector(
        SelectorType::LinearRank,
        SelectorConfig::LinearRank(LinearRankSelectorMapper),
    );
    registry.register_selector(
        SelectorType::NSGA2,
        SelectorConfig::NSGA2(NSGA2SelectorMapper),
    );
}
