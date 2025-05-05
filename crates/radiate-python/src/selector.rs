use pyo3::{pyclass, pymethods};
use radiate::{
    Chromosome, EliteSelector, GeneticEngineBuilder, RankSelector, RouletteSelector,
    TournamentSelector,
};
use std::collections::BTreeMap;

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PySelector {
    pub name: String,
    pub args: BTreeMap<String, String>,
}

#[pymethods]
impl PySelector {
    #[new]
    #[pyo3(signature = (name, args=None))]
    pub fn new(name: String, args: Option<BTreeMap<String, String>>) -> Self {
        Self {
            name,
            args: args.unwrap_or_default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_args(&self) -> &BTreeMap<String, String> {
        &self.args
    }
}

pub(crate) fn set_selector<C, T>(
    builder: GeneticEngineBuilder<C, T>,
    selector: PySelector,
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
    }

    builder
}
