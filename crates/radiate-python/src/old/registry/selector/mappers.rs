use crate::{ParamMapper, PyEngineParam};
use radiate::{
    BoltzmannSelector, EliteSelector, LinearRankSelector, NSGA2Selector, RankSelector,
    RouletteSelector, StochasticUniversalSamplingSelector, TournamentSelector,
};

pub struct TournamentSelectorMapper;

impl ParamMapper for TournamentSelectorMapper {
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
impl ParamMapper for RouletteSelectorMapper {
    type Output = RouletteSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        RouletteSelector::new()
    }
}

pub struct RankSelectorMapper;
impl ParamMapper for RankSelectorMapper {
    type Output = RankSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        RankSelector::new()
    }
}

pub struct EliteSelectorMapper;
impl ParamMapper for EliteSelectorMapper {
    type Output = EliteSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        EliteSelector::new()
    }
}

pub struct BoltzmannSelectorMapper;
impl ParamMapper for BoltzmannSelectorMapper {
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
impl ParamMapper for StochasticUniversalSamplingSelectorMapper {
    type Output = StochasticUniversalSamplingSelector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        StochasticUniversalSamplingSelector::new()
    }
}

pub struct LinearRankSelectorMapper;
impl ParamMapper for LinearRankSelectorMapper {
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
impl ParamMapper for NSGA2SelectorMapper {
    type Output = NSGA2Selector;

    fn map(&self, _: &PyEngineParam) -> Self::Output {
        NSGA2Selector::new()
    }
}
