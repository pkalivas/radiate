import pytest
import radiate as rd


@pytest.mark.unit
def test_metric_min_projection(simple_metric_set):
    expr = rd.select("one").min().cast(rd.UInt64).debug()
    assert simple_metric_set.project(expr) == 0.0


@pytest.mark.unit
def test_metric_max_projection(simple_metric_set):
    expr = rd.select("one").max()
    assert simple_metric_set.project(expr) == 9


@pytest.mark.unit
def test_metric_mean_projection(simple_metric_set):
    expr = rd.select("one").mean()
    assert simple_metric_set.project(expr) == pytest.approx(4.5)


@pytest.mark.unit
def test_metric_sum_projection(simple_metric_set):
    expr = rd.select("one").sum()
    assert simple_metric_set.project(expr) == 45


@pytest.mark.unit
def test_metric_count_projection(simple_metric_set):
    expr = rd.select("one").count()
    assert simple_metric_set.project(expr) == 10


@pytest.mark.unit
def test_when_then_expr_false_branch(simple_metric_set):
    expr = (
        rd.when(rd.select("one").min() < -1.0)
        .then(rd.select("two").mean())
        .otherwise(123123)
    )

    assert simple_metric_set.project(expr) == 123123


@pytest.mark.unit
def test_when_then_expr_true_branch(simple_metric_set):
    expr = (
        rd.when(rd.select("one").min() >= 0.0)
        .then(rd.select("two").mean())
        .otherwise(-1)
    )

    assert simple_metric_set.project(expr) == pytest.approx(14.5)


@pytest.mark.unit
def test_when_then_expr_with_literal_then(simple_metric_set):
    expr = (
        rd.when(rd.select("one").max().cast(rd.Float64) == 9.0).then(111).otherwise(222)
    )
    assert simple_metric_set.project(expr) == 111


@pytest.mark.unit
def test_when_then_expr_with_literal_otherwise(simple_metric_set):
    expr = rd.when(rd.select("one").max() < 0).then(111).otherwise(222)
    assert simple_metric_set.project(expr) == 222


@pytest.mark.unit
def test_nested_when_then(simple_metric_set):
    inner = rd.when(rd.select("two").mean() > 14.0).then(1).otherwise(2)
    outer = (
        rd.when(rd.select("one").max().cast(rd.Float64) == 9.0).then(inner).otherwise(3)
    )
    assert simple_metric_set.project(outer) == 1
