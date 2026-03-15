import radiate as rd
import pytest


@pytest.mark.integration
def test_generation_metrics(random_seed):
    num_genes = 5

    metrics = (
        (
            rd.Engine.int(num_genes, init_range=(0, 10))
            .fitness(lambda x: sum(x))
            .minimizing()
            .limit(rd.Limit.score(0), rd.Limit.generations(500))
        )
        .run()
        .metrics()
    )

    assert len(metrics) == 33
    assert len(metrics.keys()) == 33
    for key in metrics.keys():
        assert key in metrics

        if key == "scores":
            assert metrics[key].mean() is not None
            assert metrics[key].stddev() is not None
            assert metrics[key].variance() is not None
            assert metrics[key].skew() is not None
            assert metrics[key].min() is not None
            assert metrics[key].max() is not None
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


@pytest.mark.integration
def test_metrics_from_events(random_seed):
    class MetricSetAssertHandler(rd.EventHandler):
        def __init__(self):
            super().__init__(rd.EventType.EPOCH_COMPLETE)

        def on_event(self, event: rd.EngineEvent) -> None:
            assert event.event_type() == rd.EventType.EPOCH_COMPLETE

            metrics = event.metrics()
            for key in metrics.keys():
                assert key in metrics
                if key == "scores":
                    assert metrics[key].mean() is not None
                    assert metrics[key].stddev() is not None
                    assert metrics[key].variance() is not None
                    assert metrics[key].skew() is not None
                    assert metrics[key].min() is not None
                    assert metrics[key].max() is not None
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

    engine = (
        rd.Engine.int(5, init_range=(0, 10))
        .fitness(lambda x: sum(x))
        .minimizing()
        .subscribe(MetricSetAssertHandler())
    )

    engine.run(rd.ScoreLimit(0), rd.GenerationsLimit(500))


@pytest.mark.integration
def test_metric_tags(random_seed):
    engine = (
        rd.Engine.int(5, init_range=(0, 10))
        .fitness(lambda x: sum(x))
        .minimizing()
        .limit(rd.Limit.score(0), rd.Limit.generations(500))
    )

    result = engine.run()

    metrics = result.metrics()

    # Check that certain metrics have expected tags
    assert "scores" in metrics
    assert "time" in metrics

    score_tags = metrics["scores"].tags()
    time_tags = metrics["time"].tags()

    assert rd.Tag.STATISTIC in score_tags
    assert rd.Tag.SCORE in score_tags
    assert rd.Tag.DISTRIBUTION in score_tags

    assert rd.Tag.TIME in time_tags

    for metric in metrics.values_by_tag(rd.Tag.ALTERER):
        if "crossover" in metric.name():
            assert rd.Tag.CROSSOVER in metric.tags()
        if "mutator" in metric.name():
            assert rd.Tag.MUTATOR in metric.tags()
