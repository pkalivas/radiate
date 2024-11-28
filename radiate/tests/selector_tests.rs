mod utilities;

#[cfg(test)]
mod selector_tests {
    use crate::utilities::population_utils;
    use radiate::objectives::{Objective, Optimize};
    use radiate::*;
    use rstest::*;

    #[rstest]
    #[case(10, Optimize::Minimize)]
    #[case(20, Optimize::Minimize)]
    #[case(30, Optimize::Minimize)]
    #[case(10, Optimize::Maximize)]
    #[case(20, Optimize::Maximize)]
    #[case(30, Optimize::Maximize)]
    fn elite_selector_selects(#[case] num: usize, #[case] optimize: Optimize) {
        let mut population = population_utils::float_population(100);
        optimize.sort(&mut population);

        let selector = EliteSelector::new();
        let selected = selector.select(&population, &Objective::Single(optimize), num);

        for i in 0..num {
            let original = population[i].score().as_ref().unwrap().as_float();
            let selected = selected[i].score().as_ref().unwrap().as_float();

            assert_eq!(original, selected);
        }

        assert_eq!(selected.len(), num);
    }

    #[rstest]
    #[case(Box::new(BoltzmannSelector::new(4.0)), Optimize::Minimize, 80)]
    #[case(Box::new(BoltzmannSelector::new(1.0)), Optimize::Minimize, 80)]
    #[case(Box::new(BoltzmannSelector::new(4.0)), Optimize::Maximize, 80)]
    #[case(Box::new(BoltzmannSelector::new(1.0)), Optimize::Maximize, 80)]
    #[case(Box::new(RouletteSelector::new()), Optimize::Minimize, 80)]
    #[case(Box::new(RouletteSelector::new()), Optimize::Maximize, 80)]
    #[case(Box::new(TournamentSelector::new(3)), Optimize::Minimize, 80)]
    #[case(Box::new(TournamentSelector::new(3)), Optimize::Maximize, 80)]
    #[case(Box::new(RankSelector::new()), Optimize::Minimize, 80)]
    #[case(Box::new(RankSelector::new()), Optimize::Maximize, 80)]
    fn test_probability_selectors_better_than_random(
        #[case] selector: Box<dyn Select<FloatChromosome>>,
        #[case] optimize: Optimize,
        #[case] count: usize,
    ) {
        let num_permutations = 1000;
        let objectives = Objective::Single(optimize);

        let mut population = population_utils::random_float_population(100);
        optimize.sort(&mut population);

        let mut better_than_random = 0;

        let monte_carlo_selector = MonteCarloSelector::new();

        for _ in 0..num_permutations {
            let selected = selector.select(&population, &objectives, count);
            let random_selected = monte_carlo_selector.select(&population, &objectives, count);

            let observed_metric = fitness_improvement_metric(&population, &selected, &objectives);
            let random_metric =
                fitness_improvement_metric(&population, &random_selected, &objectives);

            if random_metric < observed_metric {
                better_than_random += 1;
            }
        }

        let percent_better_than_random = better_than_random as f32 / num_permutations as f32;

        assert!(percent_better_than_random > 0.95);
    }

    fn fitness_improvement_metric(
        population: &Population<FloatChromosome>,
        selected: &Population<FloatChromosome>,
        objectives: &Objective,
    ) -> f32 {
        let population_avg: f32 = population
            .iter()
            .map(|ind| ind.genotype[0].genes[0].allele)
            .sum::<f32>()
            / population.len() as f32;

        let selected_avg: f32 = selected
            .iter()
            .map(|ind| ind.genotype[0].genes[0].allele)
            .sum::<f32>()
            / selected.len() as f32;

        if let Objective::Single(optimize) = objectives {
            match optimize {
                Optimize::Minimize => population_avg - selected_avg,
                Optimize::Maximize => selected_avg - population_avg,
            }
        } else {
            panic!("Objective must be single");
        }
    }
}
