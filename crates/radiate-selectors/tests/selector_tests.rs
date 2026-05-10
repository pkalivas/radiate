mod utilities;

#[cfg(test)]
mod selector_tests {
    use crate::utilities::population_utils;
    use radiate_core::*;
    use radiate_selectors::*;
    use rstest::*;

    fn fitness_improvement_metric(
        population: &Population<FloatChromosome<f32>>,
        selected: &Population<FloatChromosome<f32>>,
        objectives: &Objective,
    ) -> f32 {
        let population_avg: f32 = population
            .iter()
            .map(|ind| ind.genotype()[0].as_slice()[0].allele())
            .sum::<f32>()
            / population.len() as f32;

        let selected_avg: f32 = selected
            .iter()
            .map(|ind| ind.genotype()[0].as_slice()[0].allele())
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
        let selected = selector.select(population.as_ref(), &Objective::Single(optimize), num);

        for i in 0..num {
            let original = population[i].score().unwrap().as_f32();
            let selected = population[selected[i]].score().unwrap().as_f32();

            assert_eq!(original, selected);
        }

        assert_eq!(selected.len(), num);
    }

    #[rstest]
    #[case(BoltzmannSelector::new(4.0), Optimize::Minimize, 80)]
    #[case(BoltzmannSelector::new(1.0), Optimize::Minimize, 80)]
    #[case(BoltzmannSelector::new(4.0), Optimize::Maximize, 80)]
    #[case(BoltzmannSelector::new(1.0), Optimize::Maximize, 80)]
    #[case(RouletteSelector::new(), Optimize::Minimize, 80)]
    #[case(RouletteSelector::new(), Optimize::Maximize, 80)]
    #[case(TournamentSelector::new(3), Optimize::Minimize, 80)]
    #[case(TournamentSelector::new(3), Optimize::Maximize, 80)]
    #[case(RankSelector::new(), Optimize::Minimize, 80)]
    #[case(RankSelector::new(), Optimize::Maximize, 80)]
    #[case(StochasticUniversalSamplingSelector::new(), Optimize::Minimize, 80)]
    #[case(StochasticUniversalSamplingSelector::new(), Optimize::Maximize, 80)]
    fn test_probability_selectors_better_than_random(
        #[case] selector: impl Select<FloatChromosome<f32>>,
        #[case] optimize: Optimize,
        #[case] count: usize,
    ) {
        let num_permutations = 1000;
        let objectives = Objective::Single(optimize);

        let mut population = population_utils::random_float_population(100);
        optimize.sort(&mut population);

        let mut better_than_random = 0;

        let random_selector = RandomSelector::new();

        for _ in 0..num_permutations {
            let selected = selector
                .select(population.as_ref(), &objectives, count)
                .into_iter()
                .map(|idx| population[idx].clone())
                .collect::<Population<FloatChromosome<f32>>>();
            let random_selected = random_selector
                .select(population.as_ref(), &objectives, count)
                .into_iter()
                .map(|idx| population[idx].clone())
                .collect::<Population<FloatChromosome<f32>>>();

            assert_eq!(selected.len(), count);
            assert_eq!(random_selected.len(), count);

            let observed_metric = fitness_improvement_metric(&population, &selected, &objectives);
            let random_metric =
                fitness_improvement_metric(&population, &random_selected, &objectives);

            if random_metric < observed_metric {
                better_than_random += 1;
            }
        }

        let percent_better_than_random = better_than_random as f32 / num_permutations as f32;

        assert!(percent_better_than_random > 0.9);
    }

    /// Every selector must return exactly `count` indices when `count <=
    /// pop.len()`.
    #[rstest]
    #[case(BoltzmannSelector::new(4.0))]
    #[case(RouletteSelector::new())]
    #[case(TournamentSelector::new(3))]
    #[case(RankSelector::new())]
    #[case(StochasticUniversalSamplingSelector::new())]
    #[case(RandomSelector::new())]
    fn selector_returns_requested_count(#[case] selector: impl Select<FloatChromosome<f32>>) {
        random_provider::scoped_seed(7, || {
            let population = population_utils::random_float_population(50);
            let objective = Objective::Single(Optimize::Maximize);

            for &count in &[1, 5, 25, 50] {
                let selected = selector.select(population.as_ref(), &objective, count);
                assert_eq!(
                    selected.len(),
                    count,
                    "selector returned {} indices, expected {count}",
                    selected.len()
                );
            }
        });
    }

    #[rstest]
    #[case(BoltzmannSelector::new(4.0))]
    #[case(RouletteSelector::new())]
    #[case(TournamentSelector::new(3))]
    #[case(RankSelector::new())]
    #[case(StochasticUniversalSamplingSelector::new())]
    fn selector_indices_in_bounds(#[case] selector: impl Select<FloatChromosome<f32>>) {
        random_provider::scoped_seed(8, || {
            let population = population_utils::random_float_population(50);
            let objective = Objective::Single(Optimize::Maximize);

            let selected = selector.select(population.as_ref(), &objective, 100);
            for idx in selected {
                assert!(
                    idx < population.len(),
                    "selector returned out-of-bounds idx {idx} for pop of size {}",
                    population.len()
                );
            }
        });
    }

    /// Tournament with k=1 degenerates to uniform random sampling — each
    /// "tournament" is a single random pick. Empirical frequency for each
    /// index should be close to 1/N. Catches bugs where k=1 accidentally
    /// short-circuits to "always pick idx 0".
    #[test]
    fn tournament_k1_approaches_uniform() {
        random_provider::scoped_seed(9, || {
            const POP_SIZE: usize = 10;
            const SAMPLES: usize = 20_000;
            const EXPECTED: f32 = SAMPLES as f32 / POP_SIZE as f32; // ~2000 per slot

            let mut population = population_utils::float_population(POP_SIZE);
            Optimize::Maximize.sort(&mut population);

            let selector = TournamentSelector::new(1);
            let objective = Objective::Single(Optimize::Maximize);

            let mut counts = vec![0usize; POP_SIZE];
            for _ in 0..SAMPLES {
                let chosen = selector.select(population.as_ref(), &objective, 1);
                counts[chosen[0]] += 1;
            }

            // 5σ bound for binomial: σ = sqrt(N * p * (1-p)) ≈ sqrt(20000 * 0.1 * 0.9) ≈ 42.
            // 5σ ≈ 210; allow generous 400 to keep the test stable.
            for (i, &c) in counts.iter().enumerate() {
                let deviation = (c as f32 - EXPECTED).abs();
                assert!(
                    deviation < 400.0,
                    "tournament k=1 skewed at idx {i}: count {c}, expected ~{EXPECTED}, deviation {deviation}"
                );
            }
        });
    }

    /// Boltzmann at very high temperature should concentrate sampling
    /// on the best individual — `temperature` controls the sharpness of
    /// the softmax over scores. Catches sign-error and weight-computation
    /// bugs that "beats random" would miss because a slightly-wrong
    /// distribution still beats random.
    #[test]
    fn boltzmann_high_temp_concentrates_on_best() {
        random_provider::scoped_seed(10, || {
            const POP_SIZE: usize = 10;
            const SAMPLES: usize = 5_000;

            let mut population = population_utils::float_population(POP_SIZE);
            // Sort descending by score so idx 0 is best for Maximize.
            Optimize::Maximize.sort(&mut population);

            let selector = BoltzmannSelector::new(50.0); // very peaked
            let objective = Objective::Single(Optimize::Maximize);

            let mut counts = vec![0usize; POP_SIZE];
            for _ in 0..SAMPLES {
                let chosen = selector.select(population.as_ref(), &objective, 1);
                counts[chosen[0]] += 1;
            }

            // The best (idx 0) should be picked >50% of the time at this temperature.
            let best_share = counts[0] as f32 / SAMPLES as f32;
            assert!(
                best_share > 0.5,
                "high-temp Boltzmann should concentrate on best; got {best_share:.3} at idx 0"
            );
        });
    }

    /// Boltzmann monotonicity: better-scoring phenotypes get sampled at
    /// least as often as worse ones
    #[test]
    fn boltzmann_monotone_in_score() {
        random_provider::scoped_seed(11, || {
            const POP_SIZE: usize = 10;
            const SAMPLES: usize = 5_000;

            let mut population = population_utils::float_population(POP_SIZE);
            // Score equals index. Sort descending so idx 0 is highest.
            Optimize::Maximize.sort(&mut population);

            let selector = BoltzmannSelector::new(2.0);
            let objective = Objective::Single(Optimize::Maximize);

            let mut counts = vec![0f32; POP_SIZE];
            for _ in 0..SAMPLES {
                let chosen = selector.select(population.as_ref(), &objective, 1);
                counts[chosen[0]] += 1.0;
            }

            // Top half (best 5) should be sampled more than bottom half.
            let top: f32 = counts[..POP_SIZE / 2].iter().sum();
            let bot: f32 = counts[POP_SIZE / 2..].iter().sum();
            assert!(
                top > bot,
                "Boltzmann not monotone: top half count {top} <= bottom half count {bot}"
            );
        });
    }

    /// Selection pressure must increase with `k`. With pop sorted
    /// descending by score, larger tournaments pick lower (better)
    /// indices on average. Catches regressions where the inner tournament
    /// loop doesn't actually reduce `best` (e.g. wrong comparison sign,
    /// off-by-one in the loop bound).
    #[test]
    fn tournament_pressure_grows_with_k() {
        random_provider::scoped_seed(12, || {
            const POP_SIZE: usize = 20;
            const SAMPLES: usize = 5_000;

            let mut population = population_utils::float_population(POP_SIZE);
            Optimize::Maximize.sort(&mut population);
            let objective = Objective::Single(Optimize::Maximize);

            let mean_idx = |k: usize| -> f32 {
                let selector = TournamentSelector::new(k);
                let total: usize = (0..SAMPLES)
                    .map(|_| selector.select(population.as_ref(), &objective, 1)[0])
                    .sum();
                total as f32 / SAMPLES as f32
            };

            let k1 = mean_idx(1);
            let k5 = mean_idx(5);
            let k10 = mean_idx(10);

            // Each step up in k should pull the mean idx lower (toward 0 = best).
            assert!(
                k5 < k1 - 1.0 && k10 < k5 - 0.5,
                "tournament not selective enough: k=1 mean {k1:.2}, k=5 mean {k5:.2}, k=10 mean {k10:.2}"
            );
        });
    }

    /// Roulette monotonicity: parallel to `boltzmann_monotone_in_score`
    /// but exercises the roulette weight transform (`scale_l1_affine_sorted`)
    /// rather than the softmax. Catches sign-error in the affine-rescale
    /// step that's specific to roulette.
    #[test]
    fn roulette_monotone_in_score() {
        random_provider::scoped_seed(13, || {
            const POP_SIZE: usize = 10;
            const SAMPLES: usize = 5_000;

            let mut population = population_utils::float_population(POP_SIZE);
            Optimize::Maximize.sort(&mut population);

            let selector = RouletteSelector::new();
            let objective = Objective::Single(Optimize::Maximize);

            let mut counts = vec![0f32; POP_SIZE];
            for _ in 0..SAMPLES {
                let chosen = selector.select(population.as_ref(), &objective, 1);
                counts[chosen[0]] += 1.0;
            }

            let top: f32 = counts[..POP_SIZE / 2].iter().sum();
            let bot: f32 = counts[POP_SIZE / 2..].iter().sum();
            assert!(
                top > bot,
                "Roulette not monotone: top half count {top} <= bottom half count {bot}"
            );
        });
    }

    /// Rank selection's defining property: distribution depends on
    /// *order*, not on absolute scores. Two pops with identical rank but
    /// wildly different score magnitudes must produce statistically
    /// indistinguishable selection counts. Catches regressions where the
    /// rank selector accidentally uses raw scores in the wheel.
    #[test]
    fn rank_ignores_score_magnitude() {
        const POP_SIZE: usize = 10;
        const SAMPLES: usize = 5_000;

        let sample = |scale: f32, seed: u64| -> Vec<usize> {
            random_provider::scoped_seed(seed, || {
                // Pop with linearly increasing scores, scaled by `scale`.
                // Both runs see the SAME rank order; only the magnitudes differ.
                let phenotypes: Vec<_> = (0..POP_SIZE)
                    .map(|i| {
                        let chrom = FloatChromosome::new(vec![FloatGene::from(0.0..1.0)]);
                        let mut p = Phenotype::from((vec![chrom], 0));
                        p.set_score(Some(Score::from((i as f32) * scale)));
                        p
                    })
                    .collect();
                let mut population = Population::new(phenotypes);
                Optimize::Maximize.sort(&mut population);

                let selector = RankSelector::new();
                let objective = Objective::Single(Optimize::Maximize);
                let mut counts = vec![0usize; POP_SIZE];
                for _ in 0..SAMPLES {
                    let chosen = selector.select(population.as_ref(), &objective, 1);
                    counts[chosen[0]] += 1;
                }
                counts
            })
        };

        // Use the same seed so the random draws are identical; only
        // the scaling differs. With identical RNG and rank-only
        // selection, counts must match exactly.
        let counts_small = sample(1e-3, 42);
        let counts_large = sample(1e3, 42);

        assert_eq!(
            counts_small, counts_large,
            "rank selector is not magnitude-invariant: {counts_small:?} vs {counts_large:?}"
        );
    }

    /// All-tied scores -> no preference for any phenotype. Selectors that
    /// silently bias toward idx 0 (e.g. tournament that always returns
    /// `best = 0` initially) skew the distribution. Each phenotype should
    /// appear in roughly `SAMPLES / POP_SIZE` of the picks.
    #[rstest]
    #[case(BoltzmannSelector::new(2.0))]
    #[case(RouletteSelector::new())]
    #[case(TournamentSelector::new(3))]
    fn equal_scores_produce_balanced_selection(
        #[case] selector: impl Select<FloatChromosome<f32>>,
    ) {
        random_provider::scoped_seed(14, || {
            const POP_SIZE: usize = 10;
            const SAMPLES: usize = 10_000;
            const EXPECTED: f32 = SAMPLES as f32 / POP_SIZE as f32;

            // Every phenotype gets the same score.
            let phenotypes: Vec<_> = (0..POP_SIZE)
                .map(|_| {
                    let chrom = FloatChromosome::new(vec![FloatGene::from(0.0..1.0)]);
                    let mut p = Phenotype::from((vec![chrom], 0));
                    p.set_score(Some(Score::from(5.0)));
                    p
                })
                .collect();
            let population = Population::new(phenotypes);
            let objective = Objective::Single(Optimize::Maximize);

            let mut counts = vec![0usize; POP_SIZE];
            for _ in 0..SAMPLES {
                let chosen = selector.select(population.as_ref(), &objective, 1);
                counts[chosen[0]] += 1;
            }

            // No bin should be more than ~3x the expected uniform count
            // — generous bound that still catches "always idx 0" bugs.
            for (i, &c) in counts.iter().enumerate() {
                assert!(
                    (c as f32) < EXPECTED * 3.0,
                    "tied-score selection skewed at idx {i}: count {c}, expected ~{EXPECTED}"
                );
            }
        });
    }
}
