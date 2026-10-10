use radiate::prelude::*;

fn main() {
    // --8<-- [start:elite_selector]
    let selector = EliteSelector::new();
    // --8<-- [end:elite_selector]

    // --8<-- [start:tournament_selector]
    // k
    let selector = TournamentSelector::new(3);
    // --8<-- [end:tournament_selector]

    // --8<-- [start:roulette_selector]
    let selector = RouletteSelector::new();
    // --8<-- [end:roulette_selector]

    // --8<-- [start:boltzmann_selector]
    // temperature
    let selector = BoltzmannSelector::new(4_f32);
    // --8<-- [end:boltzmann_selector]

    // --8<-- [start:nsga2_selector]
    let selector = NSGA2Selector::new();
    // --8<-- [end:nsga2_selector]

    // --8<-- [start:nsga3_selector]
    // partitions (reference points)
    let selector = NSGA3Selector::new(12);
    // --8<-- [end:nsga3_selector]

    // --8<-- [start:tournament_nsga2_selector]
    let selector = TournamentNSGA2Selector::new();
    // --8<-- [end:tournament_nsga2_selector]

    // --8<-- [start:stochastic_sampling_selector]
    let selector = StochasticUniversalSamplingSelector::new();
    // --8<-- [end:stochastic_sampling_selector]

    // --8<-- [start:rank_selector]
    let selector = RankSelector::new();
    // --8<-- [end:rank_selector]

    // --8<-- [start:linear_rank_selector]
    // pressure
    let selector = LinearRankSelector::new(0.1);
    // --8<-- [end:linear_rank_selector]

    // --8<-- [start:random_selector]
    let selector = RandomSelector::new();
    // --8<-- [end:random_selector]
}
