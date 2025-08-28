import pytest
import numpy as np
import radiate as rd


class TestSelectorOperators:
    """Unit tests for selector operators."""

    @pytest.mark.unit
    def test_selector_correlation(self, random_seed):
        """Test correlation between selector probabilities and expected probabilities."""
        selectors = [
            (rd.RouletteSelector(), "max"),
            (rd.RouletteSelector(), "min"),
            (rd.BoltzmannSelector(temp=2.0), "max"),
            (rd.BoltzmannSelector(temp=2.0), "min"),
        ]

        for selector, opt in selectors:
            codec = rd.FloatCodec.vector(length=5, init_range=(0.0, 1.0))

            pop_size = 100
            population = rd.Population(
                rd.Phenotype(codec.encode(), score=float(i + 1))
                for i in range(pop_size)
            )

            num_trials = 10_000
            selection_counts = np.zeros(pop_size)

            for _ in range(num_trials):
                selected = selector.select(population, opt, 1)
                idx = int(selected[0].score()[0]) - 1  # score was i+1
                selection_counts[idx] += 1

            probabilities = selection_counts / num_trials
            expected = np.array([i + 1 for i in range(pop_size)], dtype=np.float32)
            expected /= expected.sum()

            corr = np.corrcoef(expected, probabilities)[0, 1]

            assert corr > 0.9, (
                f"Correlation too low: {corr} for {selector.component} with objective {opt}"
            )

    @pytest.mark.unit
    def test_selector_empirical_bias(self, random_seed):
        """Test that selectors bias toward higher-scoring individuals as expected."""
        codec = rd.FloatCodec.vector(length=5, init_range=(0.0, 1.0))
        pop_size = 100
        population = rd.Population(
            rd.Phenotype(codec.encode(), score=float(i + 1)) for i in range(pop_size)
        )

        num_trials = 5000
        selectors = [
            (rd.TournamentSelector(k=3), "max"),
            (rd.TournamentSelector(k=3), "min"),
            (rd.RankSelector(), "max"),
            (rd.RankSelector(), "min"),
        ]

        for selector, opt in selectors:
            counts = np.zeros(pop_size)

            for _ in range(num_trials):
                selected = selector.select(population, opt, 1)
                score = selected[0].score()[0]
                idx = int(score) - 1
                counts[idx] += 1

            probs = counts / num_trials
            ranking = np.argsort(probs) if opt == "max" else np.argsort(-probs)
            top_indices = ranking[:5]

            if opt == "max":
                assert all(top_indices >= pop_size * 0.8), (
                    f"Selector {selector.component} did not favor higher scores for {opt} objective."
                )
            else:
                assert all(top_indices < pop_size * 0.2), (
                    f"Selector {selector.component} did not favor lower scores for {opt} objective."
                )
