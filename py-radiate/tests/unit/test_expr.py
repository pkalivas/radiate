import pytest
import radiate as rd


@pytest.mark.unit
def test_literal_expr_apply():
    expr = rd.lit(42)
    assert expr.apply(999) == 42


@pytest.mark.unit
def test_element_expr_apply():
    expr = rd.element()
    assert expr.apply(123) == 123
    assert expr.apply("abc") == "abc"


@pytest.mark.unit
def test_expr_add_literal():
    expr = rd.element() + 2
    assert expr.apply(3) == 5


@pytest.mark.unit
def test_expr_sub_literal():
    expr = rd.element() - 2
    assert expr.apply(7) == 5


@pytest.mark.unit
def test_expr_mul_literal():
    expr = rd.element() * 3
    assert expr.apply(4) == 12


@pytest.mark.unit
def test_expr_div_literal():
    expr = rd.element() / 2
    assert expr.apply(8) == 4


@pytest.mark.unit
def test_expr_pow_literal():
    expr = rd.element() ** 3
    assert expr.apply(2) == 8


@pytest.mark.unit
def test_expr_nested_arithmetic():
    expr = ((rd.element() + 2) * 3) - 1
    assert expr.apply(4) == 17


@pytest.mark.unit
def test_expr_lt():
    expr = rd.element() < 5
    assert expr.apply(4) is True
    assert expr.apply(5) is False


@pytest.mark.unit
def test_expr_lte():
    expr = rd.element() <= 5
    assert expr.apply(4) is True
    assert expr.apply(5) is True
    assert expr.apply(6) is False


@pytest.mark.unit
def test_expr_gt():
    expr = rd.element() > 5
    assert expr.apply(6) is True
    assert expr.apply(5) is False


@pytest.mark.unit
def test_expr_eq_literal():
    expr = rd.element() == 10
    assert expr.apply(10) is True
    assert expr.apply(9) is False


@pytest.mark.unit
def test_expr_ne_literal():
    expr = rd.element() != 10
    assert expr.apply(10) is False
    assert expr.apply(9) is True


@pytest.mark.unit
def test_expr_and():
    expr = (rd.element() > 0) & (rd.element() < 10)
    assert expr.apply(5) is True
    assert expr.apply(0) is False
    assert expr.apply(10) is False


@pytest.mark.unit
def test_expr_or():
    expr = (rd.element() < 0) | (rd.element() > 10)
    assert expr.apply(-1) is True
    assert expr.apply(11) is True
    assert expr.apply(5) is False


@pytest.mark.unit
def test_expr_neg():
    expr = -rd.element()
    assert expr.apply(3) == -3


@pytest.mark.unit
def test_expr_abs():
    expr = abs(rd.element())
    assert expr.apply(-7) == 7
    assert expr.apply(7) == 7


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
def test_metric_unique_projection():
    expr = rd.lit(5).rolling(10).unique().count()

    for i in range(10):
        expr.apply(i)

    assert expr.apply(10) == 1  # Only one unique value (5) in the last 10


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


# -----------------------------
# every / then / otherwise tests
# -----------------------------


@pytest.mark.unit
def test_every_then_otherwise_basic():
    expr = rd.every(2).then(1).otherwise(-1)

    for i in range(10):
        result = expr.apply(i)

        if i % 2 == 0:
            assert result == -1, f"Expected -1 but got {result} at i={i}"
        else:
            assert result == 1, f"Expected 1 but got {result} at i={i}"


@pytest.mark.unit
def test_every_three():
    expr = rd.every(3).then("hit").otherwise("miss")

    expected = {
        0: "miss",
        1: "miss",
        2: "hit",
        3: "miss",
        4: "miss",
        5: "hit",
    }

    for i, exp in expected.items():
        assert expr.apply(i) == exp


# -----------------------------
# clamp / cast tests
# -----------------------------


@pytest.mark.unit
def test_expr_clamp_literal_bounds():
    expr = rd.element().clamp(0, 10)
    assert expr.apply(-5) == 0
    assert expr.apply(5) == 5
    assert expr.apply(15) == 10


# -----------------------------
# error handling tests
# -----------------------------


@pytest.mark.unit
def test_add_invalid_type_raises():
    expr = rd.element()
    with pytest.raises(TypeError):
        expr.add(object())  # type: ignore[arg-type]


@pytest.mark.unit
def test_lt_invalid_type_raises():
    expr = rd.element()
    with pytest.raises(TypeError):
        expr.lt(object())  # type: ignore[arg-type]


@pytest.mark.unit
def test_eq_invalid_type_raises():
    expr = rd.element()
    with pytest.raises(TypeError):
        expr.eq(object())  # type: ignore[arg-type]
