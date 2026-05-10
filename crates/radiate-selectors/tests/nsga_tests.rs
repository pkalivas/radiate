#[cfg(test)]
mod nsga_tests {
    use radiate_test::*;
    use radiate_core::*;
    use radiate_selectors::nsga3::{
        ObjectiveBounds, fronts_from_ranks, nearest_reference_direction, niching_fill,
        to_minimization_space,
    };
    use radiate_selectors::*;

    fn min2() -> Objective {
        Objective::Multi(vec![Optimize::Minimize, Optimize::Minimize])
    }

    fn min3() -> Objective {
        Objective::Multi(vec![
            Optimize::Minimize,
            Optimize::Minimize,
            Optimize::Minimize,
        ])
    }

    // 6-point 2-objective population with a known rank structure (minimize both):
    //   rank 0: [1,3], [2,2], [3,1]
    //   rank 1: [2,3], [3,2]       (each dominated by one rank-0 point)
    //   rank 2: [3,3]               (dominated by both rank-1 points)
    fn known_rank_population_2obj() -> radiate_core::Population<FloatChromosome<f32>> {
        multi_obj_population(vec![
            vec![1.0, 3.0], // rank 0
            vec![2.0, 2.0], // rank 0
            vec![3.0, 1.0], // rank 0
            vec![2.0, 3.0], // rank 1
            vec![3.0, 2.0], // rank 1
            vec![3.0, 3.0], // rank 2
        ])
    }

    // -----------------------------------------------------------------------
    // NSGA2Selector
    // -----------------------------------------------------------------------

    #[test]
    fn nsga2_returns_correct_count() {
        let population = known_rank_population_2obj();
        let selector = NSGA2Selector::new();
        for count in [1, 3, 5, 6] {
            let selected = selector.select(population.as_ref(), &min2(), count);
            assert_eq!(selected.len(), count, "count={count}");
        }
    }

    #[test]
    fn nsga2_rank0_always_selected_before_dominated() {
        let population = known_rank_population_2obj();
        let selector = NSGA2Selector::new();

        // Selecting exactly 3 must yield the three rank-0 points.
        let selected = selector.select(population.as_ref(), &min2(), 3);
        assert_eq!(selected.len(), 3);

        let rank0: Vec<Vec<f32>> = vec![vec![1.0, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
        for &ind in selected.iter() {
            let score = population[ind].score().unwrap().as_slice().to_vec();
            assert!(
                rank0.contains(&score),
                "dominated individual {:?} was selected instead of a rank-0 point",
                score
            );
        }
    }

    #[test]
    fn nsga2_crowding_distance_breaks_ties_within_front() {
        // All 5 points are rank-0 (no pair dominates the other in min-min).
        // Boundary points [0,10] and [10,0] get infinite crowding distance.
        // Interior distances: [4,6] ~1.0, [6,4] ~1.0, [5,5] ~0.4.
        // Selecting 3 must exclude [5,5] (lowest crowding distance).
        let population = multi_obj_population(vec![
            vec![0.0, 10.0],
            vec![10.0, 0.0],
            vec![4.0, 6.0],
            vec![6.0, 4.0],
            vec![5.0, 5.0],
        ]);
        let selector = NSGA2Selector::new();
        let selected = selector.select(population.as_ref(), &min2(), 3);

        let scores: Vec<Vec<f32>> = selected
            .iter()
            .map(|&ind| population[ind].score().unwrap().as_slice().to_vec())
            .collect();

        assert!(
            !scores.contains(&vec![5.0, 5.0]),
            "expected [5,5] to be excluded (lowest crowding distance), got {:?}",
            scores
        );
    }

    #[test]
    fn nsga2_mixed_objectives_respects_dominance_direction() {
        // obj0: Minimize, obj1: Maximize.
        // [1,10] strictly dominates [2,9] (lower on obj0 AND higher on obj1).
        // [2,9]  strictly dominates [5,5].
        let objective = Objective::Multi(vec![Optimize::Minimize, Optimize::Maximize]);
        let population = multi_obj_population(vec![
            vec![1.0, 10.0], // rank 0
            vec![2.0, 9.0],  // rank 1
            vec![5.0, 5.0],  // rank 2
        ]);
        let selector = NSGA2Selector::new();
        let selected = selector.select(population.as_ref(), &objective, 1);

        assert_eq!(
            population[selected[0]].score().unwrap().as_slice(),
            &[1.0, 10.0],
            "rank-0 point must be selected first"
        );
    }

    // -----------------------------------------------------------------------
    // TournamentNSGA2Selector
    // -----------------------------------------------------------------------

    #[test]
    fn tournament_nsga2_returns_correct_count() {
        let population = known_rank_population_2obj();
        let selector = TournamentNSGA2Selector::new();
        for count in [1, 3, 6] {
            let selected = selector.select(population.as_ref(), &min2(), count);
            assert_eq!(selected.len(), count, "count={count}");
        }
    }

    #[test]
    fn tournament_nsga2_dominant_always_wins_direct_matchup() {
        // Two-individual population: the tournament k=2 always uses both,
        // so the comparison is always rank-0 vs rank-1, which is deterministic.
        let population = multi_obj_population(vec![
            vec![1.0, 1.0], // rank 0
            vec![3.0, 3.0], // rank 1: dominated on both dims
        ]);
        let selector = TournamentNSGA2Selector::new();

        for _ in 0..20 {
            let selected = selector.select(population.as_ref(), &min2(), 1);
            assert_eq!(
                population[selected[0]].score().unwrap().as_slice(),
                &[1.0, 1.0],
                "rank-0 individual must always win the tournament"
            );
        }
    }

    // -----------------------------------------------------------------------
    // NSGA3Selector
    // -----------------------------------------------------------------------

    #[test]
    fn nsga3_returns_correct_count() {
        // 3-objective population. Use enough individuals that the selector
        // has to apply niching to fill the partial front.
        let population = multi_obj_population(vec![
            vec![1.0, 0.0, 0.0], // rank 0
            vec![0.0, 1.0, 0.0], // rank 0
            vec![0.0, 0.0, 1.0], // rank 0
            vec![0.5, 0.5, 0.5], // dominated
            vec![0.8, 0.8, 0.8], // dominated
        ]);
        let selector = NSGA3Selector::new(4);

        for count in [2, 3, 4, 5] {
            let selected = selector.select(population.as_ref(), &min3(), count);
            assert_eq!(selected.len(), count, "count={count}");
        }
    }

    #[test]
    fn nsga3_rank0_always_selected_before_dominated() {
        let population = multi_obj_population(vec![
            vec![1.0, 2.0, 3.0], // rank 0
            vec![2.0, 1.0, 3.0], // rank 0
            vec![3.0, 2.0, 1.0], // rank 0
            vec![5.0, 5.0, 5.0], // rank 1+
            vec![6.0, 6.0, 6.0], // rank 1+
        ]);
        let selector = NSGA3Selector::new(4);
        let selected = selector.select(population.as_ref(), &min3(), 3);

        let dominated: Vec<Vec<f32>> = vec![vec![5.0, 5.0, 5.0], vec![6.0, 6.0, 6.0]];
        for &ind in selected.iter() {
            let score = population[ind].score().unwrap().as_slice().to_vec();
            assert!(
                !dominated.contains(&score),
                "dominated individual {:?} was selected",
                score
            );
        }
    }

    #[test]
    fn nsga3_niching_prefers_underrepresented_reference_direction() {
        // 2 ref dirs: along x-axis and along y-axis.
        // already_selected contains a point closest to the x-axis direction
        // (niche 0 count = 1). Candidates: one near x-axis (niche 0) and one
        // near y-axis (niche 1, count = 0). Niching must prefer niche 1.
        //
        // Scores are in minimization space (passed directly to niching_fill).
        let scores = vec![
            vec![1.0f32, 0.0], // idx 0 — already selected, maps to niche 0
            vec![0.9f32, 0.1], // idx 1 — candidate, maps to niche 0
            vec![0.1f32, 0.9], // idx 2 — candidate, maps to niche 1
        ];
        let ref_dirs = vec![
            vec![1.0f32, 0.0], // niche 0: x-axis
            vec![0.0f32, 1.0], // niche 1: y-axis
        ];

        let result = niching_fill(&scores, &ref_dirs, &[0], &[1, 2], 1);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0], 2,
            "niching must prefer the candidate in the underrepresented niche (y-axis)"
        );
    }

    // -----------------------------------------------------------------------
    // fronts_from_ranks
    // -----------------------------------------------------------------------

    #[test]
    fn fronts_from_ranks_empty_input() {
        assert!(fronts_from_ranks(&[]).is_empty());
    }

    #[test]
    fn fronts_from_ranks_all_rank_zero() {
        let fronts = fronts_from_ranks(&[0, 0, 0]);
        assert_eq!(fronts.len(), 1);
        assert_eq!(fronts[0].len(), 3);
        assert!(fronts[0].contains(&0) && fronts[0].contains(&1) && fronts[0].contains(&2));
    }

    #[test]
    fn fronts_from_ranks_multiple_fronts() {
        // indices 0 and 2 → rank 0; index 1 → rank 1; index 3 → rank 2
        let fronts = fronts_from_ranks(&[0, 1, 0, 2]);
        assert_eq!(fronts.len(), 3);
        assert!(fronts[0].contains(&0) && fronts[0].contains(&2));
        assert_eq!(fronts[0].len(), 2);
        assert_eq!(fronts[1], vec![1]);
        assert_eq!(fronts[2], vec![3]);
    }

    #[test]
    fn fronts_from_ranks_no_trailing_empty_fronts() {
        // Ranks 0 and 2 but no rank 1: the result must not have an empty middle entry.
        // fronts_from_ranks allocates by max rank, so there will be an empty slot at index 1.
        // The function only strips trailing empties.
        let fronts = fronts_from_ranks(&[0, 2]);
        assert_eq!(fronts.len(), 3);
        assert!(fronts[1].is_empty()); // gap in the middle is kept
        assert!(!fronts[2].is_empty());
    }

    // -----------------------------------------------------------------------
    // to_minimization_space
    // -----------------------------------------------------------------------

    #[test]
    fn to_minimization_space_minimize_is_unchanged() {
        let obj = Objective::Multi(vec![Optimize::Minimize, Optimize::Minimize]);
        assert_eq!(to_minimization_space(&[3.0, 4.0], &obj), vec![3.0, 4.0]);
    }

    #[test]
    fn to_minimization_space_maximize_is_negated() {
        let obj = Objective::Multi(vec![Optimize::Maximize, Optimize::Maximize]);
        assert_eq!(to_minimization_space(&[3.0, 4.0], &obj), vec![-3.0, -4.0]);
    }

    #[test]
    fn to_minimization_space_mixed_objectives() {
        let obj = Objective::Multi(vec![Optimize::Minimize, Optimize::Maximize]);
        assert_eq!(to_minimization_space(&[3.0, 4.0], &obj), vec![3.0, -4.0]);
    }

    // -----------------------------------------------------------------------
    // ObjectiveBounds
    // -----------------------------------------------------------------------

    #[test]
    fn objective_bounds_empty_input() {
        // Empty scores → empty bounds; normalize on an empty point round-trips empty.
        let bounds = ObjectiveBounds::from_scores(&[]);
        assert!(bounds.normalize(&[]).is_empty());
    }

    #[test]
    fn objective_bounds_single_point_is_degenerate() {
        // Single point → ideal == nadir on every dim → normalize must return zeros.
        let bounds = ObjectiveBounds::from_scores(&[vec![2.0f32, 5.0]]);
        assert_eq!(bounds.normalize(&[2.0, 5.0]), vec![0.0, 0.0]);
    }

    #[test]
    fn objective_bounds_known_values_normalize_correctly() {
        // ideal = [1, 2], nadir = [3, 5]. The corners map to 0 and 1 respectively,
        // which exercises that from_scores computed both bounds correctly.
        let scores = vec![vec![1.0f32, 4.0], vec![3.0, 2.0], vec![2.0, 5.0]];
        let bounds = ObjectiveBounds::from_scores(&scores);

        assert_eq!(bounds.normalize(&[1.0, 2.0]), vec![0.0, 0.0]);
        assert_eq!(bounds.normalize(&[3.0, 5.0]), vec![1.0, 1.0]);
    }

    #[test]
    fn objective_bounds_normalize_known_values() {
        // ideal = [0, 0], nadir = [10, 10] → [5, 2] → [0.5, 0.2].
        let bounds = ObjectiveBounds::from_scores(&[vec![0.0f32, 0.0], vec![10.0, 10.0]]);
        let result = bounds.normalize(&[5.0, 2.0]);

        assert!((result[0] - 0.5).abs() < 1e-5);
        assert!((result[1] - 0.2).abs() < 1e-5);
    }

    #[test]
    fn objective_bounds_zero_range_dimension_is_zero() {
        // dim 0 has ideal == nadir (every point has 5.0 on dim 0) → degenerate, must output 0.0.
        let bounds = ObjectiveBounds::from_scores(&[vec![5.0f32, 0.0], vec![5.0, 10.0]]);
        let result = bounds.normalize(&[5.0, 7.0]);

        assert_eq!(result[0], 0.0);
        assert!((result[1] - 0.7).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // nearest_reference_direction
    // -----------------------------------------------------------------------

    #[test]
    fn nearest_reference_direction_picks_closest() {
        let ref_dirs = vec![
            vec![1.0f32, 0.0], // x-axis
            vec![0.0f32, 1.0], // y-axis
        ];
        // Point mostly along x-axis → should associate with direction 0.
        let (k, _) = nearest_reference_direction(&[0.9, 0.1], &ref_dirs);
        assert_eq!(k, 0, "expected x-axis direction");

        // Point mostly along y-axis → should associate with direction 1.
        let (k, _) = nearest_reference_direction(&[0.1, 0.9], &ref_dirs);
        assert_eq!(k, 1, "expected y-axis direction");
    }

    #[test]
    fn nearest_reference_direction_exact_alignment_has_zero_distance() {
        let ref_dirs = vec![vec![1.0f32, 0.0], vec![0.0f32, 1.0]];
        let (k, d) = nearest_reference_direction(&[1.0, 0.0], &ref_dirs);
        assert_eq!(k, 0);
        assert!(d < 1e-5, "exact alignment must have near-zero distance");
    }

    // -----------------------------------------------------------------------
    // niching_fill
    // -----------------------------------------------------------------------

    #[test]
    fn niching_fill_zero_remaining_is_empty() {
        let scores = vec![vec![0.0f32, 1.0]];
        let ref_dirs = vec![vec![1.0f32, 0.0]];
        let result = niching_fill(&scores, &ref_dirs, &[], &[0], 0);
        assert!(result.is_empty());
    }

    #[test]
    fn niching_fill_returns_correct_count() {
        let scores = vec![
            vec![1.0f32, 0.0],
            vec![0.5, 0.5],
            vec![0.0, 1.0],
            vec![0.8, 0.2],
            vec![0.2, 0.8],
        ];
        let ref_dirs = vec![vec![1.0f32, 0.0], vec![0.5, 0.5], vec![0.0, 1.0]];
        let result = niching_fill(&scores, &ref_dirs, &[0, 2], &[1, 3, 4], 2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn niching_fill_prefers_empty_niche_over_occupied() {
        // Niche 0 already has 1 member (idx 0). Niche 1 is empty.
        // Candidate idx 1 maps to niche 0; candidate idx 2 maps to niche 1.
        // Niching must pick idx 2.
        let scores = vec![
            vec![1.0f32, 0.0], // idx 0 — already selected → niche 0 count = 1
            vec![0.9f32, 0.1], // idx 1 — candidate → niche 0
            vec![0.1f32, 0.9], // idx 2 — candidate → niche 1
        ];
        let ref_dirs = vec![vec![1.0f32, 0.0], vec![0.0f32, 1.0]];

        let result = niching_fill(&scores, &ref_dirs, &[0], &[1, 2], 1);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0], 2,
            "should prefer candidate in empty niche (niche 1) over occupied niche (niche 0)"
        );
    }
}
