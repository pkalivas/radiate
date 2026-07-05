import pytest

import radiate as rd


@pytest.mark.unit
def test_metric_min_projection(simple_metric_set):
    expr = rd.Expr.select("one").min().cast(rd.UInt64).debug()
    assert expr.eval(simple_metric_set) == 0.0


@pytest.mark.unit
def test_metric_max_projection(simple_metric_set):
    expr = rd.Expr.select("one").max()
    assert expr.eval(simple_metric_set) == 9


@pytest.mark.unit
def test_metric_mean_projection(simple_metric_set):
    expr = rd.Expr.select("one").mean()
    assert expr.eval(simple_metric_set) == pytest.approx(4.5)


@pytest.mark.unit
def test_metric_sum_projection(simple_metric_set):
    expr = rd.Expr.select("one").sum()
    assert expr.eval(simple_metric_set) == 45


@pytest.mark.unit
def test_metric_count_projection(simple_metric_set):
    expr = rd.Expr.select("one").count()
    assert expr.eval(simple_metric_set) == 10


@pytest.mark.unit
def test_when_then_expr_false_branch(simple_metric_set):
    expr = (
        rd.Expr.when(rd.Expr.select("one").min() < -1.0)
        .then(rd.Expr.select("two").mean())
        .otherwise(123123)
    )

    assert expr.eval(simple_metric_set) == 123123


@pytest.mark.unit
def test_when_then_expr_true_branch(simple_metric_set):
    expr = (
        rd.Expr.when(rd.Expr.select("one").min() >= 0.0)
        .then(rd.Expr.select("two").mean())
        .otherwise(-1)
    )

    assert expr.eval(simple_metric_set) == pytest.approx(14.5)


@pytest.mark.unit
def test_when_then_expr_with_literal_then(simple_metric_set):
    expr = (
        rd.Expr.when(rd.Expr.select("one").max().cast(rd.Float64) == 9.0)
        .then(111)
        .otherwise(222)
    )
    assert expr.eval(simple_metric_set) == 111


@pytest.mark.unit
def test_when_then_expr_with_literal_otherwise(simple_metric_set):
    expr = rd.Expr.when(rd.Expr.select("one").max() < 0).then(111).otherwise(222)
    assert expr.eval(simple_metric_set) == 222


@pytest.mark.unit
def test_nested_when_then(simple_metric_set):
    inner = rd.Expr.when(rd.Expr.select("two").mean() > 14.0).then(1).otherwise(2)
    outer = (
        rd.Expr.when(rd.Expr.select("one").max().cast(rd.Float64) == 9.0)
        .then(inner)
        .otherwise(3)
    )
    assert outer.eval(simple_metric_set) == 1


@pytest.mark.unit
def test_constructors_live_on_expr_namespace():
    for name in ("select", "lit", "when", "every", "generation", "element"):
        assert not hasattr(rd, name), f"rd.{name} should no longer exist"
    for name in (
        "select",
        "lit",
        "when",
        "every",
        "generation",
        "element",
        "error",
        "stagnation",
        "is_stagnant",
    ):
        assert hasattr(rd.Expr, name), f"rd.Expr.{name} should exist"


@pytest.mark.unit
def test_lit_and_generation_constructors(simple_metric_set):
    assert rd.Expr.lit(7.0).eval(simple_metric_set) == pytest.approx(7.0)
    # generation() reads the "index" metric; just confirm it builds and projects.
    rd.Expr.generation().eval(simple_metric_set)
