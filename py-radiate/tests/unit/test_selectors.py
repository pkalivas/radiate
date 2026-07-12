import numpy as np
import pytest

import radiate as rd


@pytest.mark.unit
def test_selector_correlation(random_seed):
    """Test correlation between selector probabilities and expected probabilities."""
    selectors = [
        (rd.Select.roulette(), rd.MAX),
        (rd.Select.roulette(), rd.MIN),
        (rd.Select.boltzmann(temp=2.0), rd.MAX),
        (rd.Select.boltzmann(temp=2.0), rd.MIN),
    ]
    codec = rd.FloatCodec(shape=5, init_range=(0.0, 1.0))
    pop_size = 100
    num_trials = 10_000

    scores = np.arange(1, pop_size + 1, dtype=np.float32)

    population = rd.Population(
        rd.Phenotype(codec.encode(), score=float(score)) for score in scores
    )

    for selector, opt in selectors:
        selection_counts = np.zeros(pop_size, dtype=np.int32)

        for _ in range(num_trials):
            selected = selector.select(population, opt, 1)
            idx = int(selected[0].score()[0]) - 1
            selection_counts[idx] += 1

        observed = selection_counts / num_trials

        if opt == rd.MAX:
            expected = scores.copy()
        else:
            # mirror typical minimization weighting
            expected = scores.max() - scores + 1

        expected /= expected.sum()
        corr = np.corrcoef(expected, observed)[0, 1]

        assert corr > 0.9, (
            f"{selector.component} ({opt}) produced weak correlation: {corr:.4f}"
        )


@pytest.mark.unit
def test_selector_empirical_bias(random_seed):
    """Test that selectors bias toward higher-scoring individuals as expected."""
    codec = rd.FloatCodec(shape=5, init_range=(0.0, 1.0))
    pop_size = 100
    population = rd.Population(
        rd.Phenotype(codec.encode(), score=float(i + 1)) for i in range(pop_size)
    )

    num_trials = 5000
    selectors = [
        (rd.Select.tournament(k=3), rd.MAX),
        (rd.Select.tournament(k=3), rd.MIN),
    ]

    for selector, opt in selectors:
        counts = np.zeros(pop_size)

        for _ in range(num_trials):
            selected = selector.select(population, opt, 1)
            score = selected[0].score()[0]
            idx = int(score) - 1
            counts[idx] += 1

        probs = counts / num_trials
        ranking = np.argsort(probs) if opt == rd.MAX else np.argsort(-probs)
        top_indices = ranking[:5]

        if opt == rd.MAX:
            assert all(top_indices >= pop_size * 0.8), (
                f"Selector {selector.component} did not favor higher scores for {opt} objective."
            )
        else:
            assert all(top_indices < pop_size * 0.2), (
                f"Selector {selector.component} did not favor lower scores for {opt} objective."
            )
