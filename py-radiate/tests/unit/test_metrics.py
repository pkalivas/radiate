import radiate as rd
import pytest


class TestMetrics:
    @pytest.mark.integration
    def test_generation_metrics(self, random_seed):
        num_genes = 5
        engine = rd.GeneticEngine(
            codec=rd.IntCodec.vector(num_genes, init_range=(0, 10)),
            fitness_func=lambda x: sum(x),
            objective="min",
        )

        result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

        metrics = result.metrics()

        assert len(metrics) == 21
        assert len(metrics.keys()) == 21
        for key in metrics.keys():
            assert key in metrics

            if key == "scores":
                assert metrics[key].seq_mean() is not None
                assert metrics[key].seq_stddev() is not None
                assert metrics[key].seq_variance() is not None
                assert metrics[key].seq_kurt() is not None
                assert metrics[key].seq_skew() is not None
                assert metrics[key].seq_min() is not None
                assert metrics[key].seq_max() is not None
                assert metrics[key].seq_count() == 100  # num phenotypes in population
                assert len(metrics[key].seq_last()) == metrics[key].seq_count()
            elif key == "time" or "step" in key:
                assert metrics[key].time_last() is not None
                assert metrics[key].time_sum() is not None
                assert metrics[key].time_mean() is not None
                assert metrics[key].time_stddev() is not None
                assert metrics[key].time_variance() is not None
                assert metrics[key].time_min() is not None
                assert metrics[key].time_max() is not None
            elif "step" not in key:
                assert metrics[key].value_last() is not None
                assert metrics[key].mean() is not None
                assert metrics[key].stddev() is not None
                assert metrics[key].variance() is not None
                assert metrics[key].skew() is not None
                assert metrics[key].min() is not None
                assert metrics[key].max() is not None
                assert metrics[key].count() is not None
