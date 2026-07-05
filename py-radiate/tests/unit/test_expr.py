import pytest

import radiate as rd

# ── fixtures ────────────────────────────────────────────────────────────────

# simple_metric_set comes from conftest.py:
#   one   = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
#   two   = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
#   const = [5.0] * 10


def _ms(val: float) -> rd.MetricSet:
    """Single-value MetricSet — used to drive generational expressions."""
    return rd.MetricSet(val=[val])


# ── basic aggregation projections ────────────────────────────────────────────


@pytest.mark.unit
def test_metric_min_projection(simple_metric_set):
    expr = rd.Expr.select("one").min().cast(rd.UInt64).debug()
    assert expr.eval(simple_metric_set) == 0.0


@pytest.mark.unit
def test_metric_max_projection(simple_metric_set):
    assert rd.Expr.select("one").max().eval(simple_metric_set) == 9


@pytest.mark.unit
def test_metric_mean_projection(simple_metric_set):
    assert rd.Expr.select("one").mean().eval(simple_metric_set) == pytest.approx(4.5)


@pytest.mark.unit
def test_metric_sum_projection(simple_metric_set):
    assert rd.Expr.select("one").sum().eval(simple_metric_set) == 45


@pytest.mark.unit
def test_metric_count_projection(simple_metric_set):
    assert rd.Expr.select("one").count().eval(simple_metric_set) == 10


@pytest.mark.unit
def test_last_value(simple_metric_set):
    # last() returns the most recently pushed observation
    assert rd.Expr.select("one").last().eval(simple_metric_set) == pytest.approx(9.0)


@pytest.mark.unit
def test_first_value_is_most_recent(simple_metric_set):
    # WindowBuffer is newest-first, so first() == last() in a single-snapshot MetricSet.
    # first() and last() become distinct when combined with rolling() across generations.
    first = rd.Expr.select("one").first().eval(simple_metric_set)
    last = rd.Expr.select("one").last().eval(simple_metric_set)
    assert first == last == pytest.approx(9.0)


@pytest.mark.unit
def test_unique_returns_distinct_values(simple_metric_set):
    # unique() returns a list of distinct values, not a count
    result = rd.Expr.select("const").unique().eval(simple_metric_set)
    assert result == [5.0]


@pytest.mark.unit
def test_stddev_zero_on_constant(simple_metric_set):
    assert rd.Expr.select("const").stddev().eval(simple_metric_set) == pytest.approx(0.0, abs=1e-5)


@pytest.mark.unit
def test_stddev_positive_on_varied(simple_metric_set):
    result = rd.Expr.select("one").stddev().eval(simple_metric_set)
    assert result > 0.0


# ── rolling window — generational accumulation ───────────────────────────────
#
# rolling(n) is a generational window: it accumulates one data point per
# eval() call and tracks the last n such calls. Testing requires calling
# eval() repeatedly with different MetricSets.


@pytest.mark.unit
def test_rolling_mean_across_evals():
    expr = rd.Expr.select("val").rolling(3).mean()
    expr.eval(_ms(1.0))
    expr.eval(_ms(2.0))
    result = expr.eval(_ms(3.0))
    # window = [3.0, 2.0, 1.0], mean = 2.0
    assert result == pytest.approx(2.0)


@pytest.mark.unit
def test_rolling_drops_oldest():
    expr = rd.Expr.select("val").rolling(3).mean()
    for v in [1.0, 2.0, 3.0]:
        expr.eval(_ms(v))
    result = expr.eval(_ms(10.0))
    # window = [10.0, 3.0, 2.0], mean = 5.0
    assert result == pytest.approx(5.0)


@pytest.mark.unit
def test_rolling_max_across_evals():
    expr = rd.Expr.select("val").rolling(3).max()
    for v in [5.0, 2.0, 8.0]:
        expr.eval(_ms(v))
    # window = [8.0, 2.0, 5.0], max = 8.0
    assert expr.eval(_ms(8.0)) == pytest.approx(8.0)


@pytest.mark.unit
def test_rolling_min_across_evals():
    expr = rd.Expr.select("val").rolling(3).min()
    for v in [5.0, 2.0, 8.0]:
        expr.eval(_ms(v))
    result = expr.eval(_ms(1.0))
    # window = [1.0, 8.0, 2.0], min = 1.0
    assert result == pytest.approx(1.0)


@pytest.mark.unit
def test_rolling_stddev_zero_on_constant():
    expr = rd.Expr.select("val").rolling(5).stddev()
    for _ in range(5):
        result = expr.eval(_ms(7.0))
    # all values the same → stddev = 0
    assert result == pytest.approx(0.0, abs=1e-5)


@pytest.mark.unit
def test_rolling_count_respects_window():
    expr = rd.Expr.select("val").rolling(4).count()
    for _ in range(10):
        count = expr.eval(_ms(1.0))
    # window is capped at 4 regardless of total evals
    assert count == 4


# ── arithmetic operators ─────────────────────────────────────────────────────


@pytest.mark.unit
def test_add_two_exprs(simple_metric_set):
    expr = rd.Expr.select("one").max() + rd.Expr.select("two").min()
    assert expr.eval(simple_metric_set) == pytest.approx(19.0)  # 9 + 10


@pytest.mark.unit
def test_add_scalar(simple_metric_set):
    expr = rd.Expr.select("one").max() + 1.0
    assert expr.eval(simple_metric_set) == pytest.approx(10.0)


@pytest.mark.unit
def test_sub_exprs(simple_metric_set):
    expr = rd.Expr.select("one").max() - rd.Expr.select("one").min()
    assert expr.eval(simple_metric_set) == pytest.approx(9.0)


@pytest.mark.unit
def test_mul_scalar(simple_metric_set):
    expr = rd.Expr.select("one").mean() * 2.0
    assert expr.eval(simple_metric_set) == pytest.approx(9.0)


@pytest.mark.unit
def test_div_exprs(simple_metric_set):
    # count() returns an integer type — cast to float before dividing
    expr = rd.Expr.select("one").sum() / rd.Expr.select("one").count().cast(rd.Float64)
    assert expr.eval(simple_metric_set) == pytest.approx(4.5)


@pytest.mark.unit
def test_div_with_literal(simple_metric_set):
    expr = rd.Expr.select("one").max() / 3.0
    assert expr.eval(simple_metric_set) == pytest.approx(3.0)


@pytest.mark.unit
def test_pow_float_exponent(simple_metric_set):
    # pow() requires float operands on both sides; int literal causes a type error
    expr = rd.Expr.lit(2.0) ** rd.Expr.lit(10.0)
    assert expr.eval(simple_metric_set) == pytest.approx(1024.0)


@pytest.mark.unit
def test_neg_unary(simple_metric_set):
    expr = -rd.Expr.select("one").max()
    assert expr.eval(simple_metric_set) == pytest.approx(-9.0)


@pytest.mark.unit
def test_abs_unary(simple_metric_set):
    # abs(min - max) = abs(0 - 9) = 9
    expr = abs(rd.Expr.select("one").min() - rd.Expr.select("one").max())
    assert expr.eval(simple_metric_set) == pytest.approx(9.0)


@pytest.mark.unit
def test_arithmetic_chain(simple_metric_set):
    # (max - min) / 10.0 = (9 - 0) / 10.0 = 0.9
    # Use a float literal to avoid int-divide panic from count()
    expr = (rd.Expr.select("one").max() - rd.Expr.select("one").min()) / 10.0
    assert expr.eval(simple_metric_set) == pytest.approx(0.9)


# ── boolean operators ────────────────────────────────────────────────────────


@pytest.mark.unit
def test_and_both_true(simple_metric_set):
    expr = (rd.Expr.select("one").min() < 1.0) & (rd.Expr.select("two").min() > 5.0)
    assert expr.eval(simple_metric_set)  # 0 < 1 and 10 > 5


@pytest.mark.unit
def test_and_one_false(simple_metric_set):
    expr = (rd.Expr.select("one").min() > 1.0) & (rd.Expr.select("two").min() > 5.0)
    assert not expr.eval(simple_metric_set)  # 0 > 1 is False


@pytest.mark.unit
def test_or_one_true(simple_metric_set):
    expr = (rd.Expr.select("one").max() > 100.0) | (rd.Expr.select("two").min() > 5.0)
    assert expr.eval(simple_metric_set)  # second branch is True


@pytest.mark.unit
def test_or_both_false(simple_metric_set):
    expr = (rd.Expr.select("one").max() > 100.0) | (rd.Expr.select("two").max() > 100.0)
    assert not expr.eval(simple_metric_set)  # 9 > 100 and 19 > 100 are both False


@pytest.mark.unit
def test_not_of_true_condition(simple_metric_set):
    expr = ~(rd.Expr.select("one").max() > 1.0)
    assert not expr.eval(simple_metric_set)  # 9 > 1 is True → ~ → False


@pytest.mark.unit
def test_not_of_false_condition(simple_metric_set):
    expr = ~(rd.Expr.select("one").max() > 100.0)
    assert expr.eval(simple_metric_set)  # 9 > 100 is False → ~ → True


@pytest.mark.unit
def test_ne_comparison_unequal(simple_metric_set):
    expr = rd.Expr.select("one").max() != rd.Expr.select("one").min()
    assert expr.eval(simple_metric_set)  # 9 != 0


@pytest.mark.unit
def test_eq_comparison_true(simple_metric_set):
    # all const values are 5.0 so min == max
    expr = rd.Expr.select("const").min() == rd.Expr.select("const").max()
    assert expr.eval(simple_metric_set)


# ── clamp ────────────────────────────────────────────────────────────────────


@pytest.mark.unit
def test_clamp_at_upper_bound(simple_metric_set):
    # mean=4.5, clamped to [0, 4] → 4.0
    expr = rd.Expr.select("one").mean().clamp(0.0, 4.0)
    assert expr.eval(simple_metric_set) == pytest.approx(4.0)


@pytest.mark.unit
def test_clamp_at_lower_bound(simple_metric_set):
    # mean=4.5, clamped to [5, 10] → 5.0
    expr = rd.Expr.select("one").mean().clamp(5.0, 10.0)
    assert expr.eval(simple_metric_set) == pytest.approx(5.0)


@pytest.mark.unit
def test_clamp_within_range(simple_metric_set):
    # mean=4.5, clamped to [0, 10] → unchanged
    expr = rd.Expr.select("one").mean().clamp(0.0, 10.0)
    assert expr.eval(simple_metric_set) == pytest.approx(4.5)


# ── error (signed relative distance from target) ─────────────────────────────


@pytest.mark.unit
def test_error_at_exact_target(simple_metric_set):
    expr = rd.Expr.select("one").mean().error(4.5)
    assert expr.eval(simple_metric_set) == pytest.approx(0.0, abs=1e-5)


@pytest.mark.unit
def test_error_below_target(simple_metric_set):
    # (4.5 - 9.0) / 9.0 = -0.5
    expr = rd.Expr.select("one").mean().error(9.0)
    assert expr.eval(simple_metric_set) == pytest.approx(-0.5)


@pytest.mark.unit
def test_error_above_target(simple_metric_set):
    # (9.0 - 4.5) / 4.5 = 1.0
    expr = rd.Expr.select("one").max().error(4.5)
    assert expr.eval(simple_metric_set) == pytest.approx(1.0)


# ── every schedule ───────────────────────────────────────────────────────────
#
# every(n) counts evaluation calls and triggers on every nth call, regardless
# of the generation index stored in the MetricSet. Requires n sequential
# evals to verify the trigger.


@pytest.mark.unit
def test_every_fires_on_nth_eval():
    ms = _ms(0.0)
    expr = rd.Expr.every(3).then(rd.Expr.lit(1.0)).otherwise(rd.Expr.lit(0.0))
    results = [expr.eval(ms) for _ in range(6)]
    # triggers on calls 3 and 6 (1-indexed)
    assert results[2] == pytest.approx(1.0)  # 3rd call
    assert results[5] == pytest.approx(1.0)  # 6th call


@pytest.mark.unit
def test_every_else_between_triggers():
    ms = _ms(0.0)
    expr = rd.Expr.every(4).then(rd.Expr.lit(1.0)).otherwise(rd.Expr.lit(0.0))
    results = [expr.eval(ms) for _ in range(8)]
    inactive = [r for i, r in enumerate(results, 1) if i % 4 != 0]
    assert all(r == pytest.approx(0.0) for r in inactive)


@pytest.mark.unit
def test_every_with_expr_branches():
    ms = rd.MetricSet(one=list(range(10)), two=list(range(10, 20)), const=[5.0] * 10)
    expr = (
        rd.Expr.every(2)
        .then(rd.Expr.select("one").max())
        .otherwise(rd.Expr.select("one").min())
    )
    expr.eval(ms)  # call 1 → else → 0.0
    result = expr.eval(ms)  # call 2 → then → 9.0
    assert result == pytest.approx(9.0)


# ── when / then / otherwise ──────────────────────────────────────────────────


@pytest.mark.unit
def test_when_then_false_branch(simple_metric_set):
    expr = (
        rd.Expr.when(rd.Expr.select("one").min() < -1.0)
        .then(rd.Expr.select("two").mean())
        .otherwise(123123)
    )
    assert expr.eval(simple_metric_set) == 123123


@pytest.mark.unit
def test_when_then_true_branch(simple_metric_set):
    expr = (
        rd.Expr.when(rd.Expr.select("one").min() >= 0.0)
        .then(rd.Expr.select("two").mean())
        .otherwise(-1)
    )
    assert expr.eval(simple_metric_set) == pytest.approx(14.5)


@pytest.mark.unit
def test_when_then_literal_then(simple_metric_set):
    expr = (
        rd.Expr.when(rd.Expr.select("one").max().cast(rd.Float64) == 9.0)
        .then(111)
        .otherwise(222)
    )
    assert expr.eval(simple_metric_set) == 111


@pytest.mark.unit
def test_when_then_literal_otherwise(simple_metric_set):
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
def test_when_with_arithmetic_in_branch(simple_metric_set):
    # mean=4.5 > 4.0, so take the then branch: mean * 2 = 9.0
    expr = (
        rd.Expr.when(rd.Expr.select("one").mean() > 4.0)
        .then(rd.Expr.select("one").mean() * 2.0)
        .otherwise(0.0)
    )
    assert expr.eval(simple_metric_set) == pytest.approx(9.0)


# ── cast ─────────────────────────────────────────────────────────────────────


@pytest.mark.unit
def test_cast_to_uint64(simple_metric_set):
    expr = rd.Expr.select("one").min().cast(rd.UInt64)
    assert expr.eval(simple_metric_set) == 0


@pytest.mark.unit
def test_cast_to_float64(simple_metric_set):
    expr = rd.Expr.select("one").max().cast(rd.Float64)
    assert expr.eval(simple_metric_set) == pytest.approx(9.0)


@pytest.mark.unit
def test_cast_enables_float_division(simple_metric_set):
    # count() returns an integer type — casting lets you use it in float arithmetic
    expr = rd.Expr.select("one").sum() / rd.Expr.select("one").count().cast(rd.Float64)
    assert expr.eval(simple_metric_set) == pytest.approx(4.5)


# ── operator overload equivalence ────────────────────────────────────────────


@pytest.mark.unit
def test_lt_overload_matches_method(simple_metric_set):
    via_op = (rd.Expr.select("one").min() < 1.0).eval(simple_metric_set)
    via_method = rd.Expr.select("one").min().lt(1.0).eval(simple_metric_set)
    assert via_op == via_method


@pytest.mark.unit
def test_add_overload_matches_method(simple_metric_set):
    via_op = (rd.Expr.select("one").max() + 1.0).eval(simple_metric_set)
    via_method = rd.Expr.select("one").max().add(1.0).eval(simple_metric_set)
    assert via_op == pytest.approx(via_method)


# ── constructor namespace ────────────────────────────────────────────────────


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
    # generation() reads "index" metric; just confirm it builds and evaluates
    rd.Expr.generation().eval(simple_metric_set)


# ── realistic usage patterns ─────────────────────────────────────────────────


@pytest.mark.unit
def test_adaptive_rate_pattern(simple_metric_set):
    """Common pattern: return a low rate normally, boost when a metric is low."""
    rate_expr = (
        rd.Expr.when(rd.Expr.select("one").mean() < 3.0)
        .then(rd.Expr.lit(0.30))
        .otherwise(rd.Expr.lit(0.05))
    )
    # mean=4.5 is not below 3.0 → low rate
    assert rate_expr.eval(simple_metric_set) == pytest.approx(0.05)


@pytest.mark.unit
def test_normalized_improvement_pattern(simple_metric_set):
    """Relative progress: (best - mean) / best."""
    best = rd.Expr.select("one").max()
    mean = rd.Expr.select("one").mean()
    progress = (best - mean) / best
    # (9 - 4.5) / 9 = 0.5
    assert progress.eval(simple_metric_set) == pytest.approx(0.5)


@pytest.mark.unit
def test_rolling_error_pattern():
    """Smoothed signed error toward a target — common diversity control pattern."""
    target = 4.0
    expr = rd.Expr.select("val").rolling(3).mean().error(target)
    for v in [2.0, 4.0, 6.0]:
        result = expr.eval(_ms(v))
    # rolling mean of [6.0, 4.0, 2.0] = 4.0; error = (4.0 - 4.0) / 4.0 = 0.0
    assert result == pytest.approx(0.0, abs=1e-5)


@pytest.mark.unit
def test_oscillating_rate_with_every():
    """every() makes a rate alternate between two values on a fixed schedule."""
    ms = rd.MetricSet(one=list(range(10)), two=list(range(10, 20)), const=[5.0] * 10)
    expr = rd.Expr.every(2).then(rd.Expr.lit(0.9)).otherwise(rd.Expr.lit(0.1))

    r1 = expr.eval(ms)
    r2 = expr.eval(ms)
    r3 = expr.eval(ms)
    r4 = expr.eval(ms)

    # calls 2 and 4 are the active ones; calls 1 and 3 are inactive
    assert r1 == pytest.approx(0.1)
    assert r2 == pytest.approx(0.9)
    assert r3 == pytest.approx(0.1)
    assert r4 == pytest.approx(0.9)
