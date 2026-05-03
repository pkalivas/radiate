from __future__ import annotations
from typing import Any

from radiate._bridge.input import RsObject
from radiate._typing import RdDataType
from radiate.radiate import PyExpr


def _coerce(value, *, allow_str: bool = False):
    if isinstance(value, Expr):
        return value.__backend__()
    types = (float, int, str) if allow_str else (float, int)
    if isinstance(value, types):
        return PyExpr.literal(value)
    raise TypeError(f"Expected Expr or scalar, got {type(value).__name__}")


def _binary_op(backend_name: str, *, allow_str: bool = False):
    def op(self, rhs):
        return Expr.from_rust(
            getattr(self.__backend__(), backend_name)(_coerce(rhs, allow_str=allow_str))
        )

    return op


class Then(RsObject):
    def __init__(self, condition: Expr | int, then_expr: Expr):
        self.condition = condition
        self.then_expr = then_expr

    def __repr__(self) -> str:
        return f"Then(condition={self.condition}, then={self.then_expr})"

    def otherwise(self, else_expr: Expr | float | int | str) -> Expr:
        else_backend = _coerce(else_expr, allow_str=True)
        if isinstance(self.condition, int):
            return Expr.from_rust(
                PyExpr.every(
                    self.condition,
                    self.then_expr.__backend__(),
                    else_backend,
                )
            )
        return Expr.from_rust(
            PyExpr.when_then_otherwise(
                condition=self.condition.__backend__(),
                then_expr=self.then_expr.__backend__(),
                else_expr=else_backend,
            )
        )


class When(RsObject):
    def __init__(self, condition: Expr):
        self.condition = condition

    def __repr__(self) -> str:
        return f"When(condition={self.condition})"

    def then(self, then_expr: Expr | float | int | str) -> Then:
        return Then(
            condition=self.condition,
            then_expr=Expr.from_rust(_coerce(then_expr, allow_str=True)),
        )


class Every(RsObject):
    def __init__(self, interval: int):
        self.interval = interval

    def __repr__(self) -> str:
        return f"Every(interval={self.interval})"

    def then(self, then_expr: Expr | float | int | str) -> Then:
        return Then(
            condition=self.interval,
            then_expr=Expr.from_rust(_coerce(then_expr, allow_str=True)),
        )


class Expr(RsObject):
    """
    Base class for all expressions in the Radiate DSL. This class serves as a wrapper around Rust
    expressions, allowing them to be used seamlessly in Python.

    Parameters
    ----------
    rs_expr
        The underlying Rust expression object that this Python `Expr` wraps.
    """

    def __repr__(self) -> str:
        return f"Expr({self.__backend__().__repr__()})"

    def __str__(self) -> str:
        return self.__backend__().__str__()

    def __lt__(self, other):
        return self.lt(other)

    def __le__(self, other):
        return self.lte(other)

    def __gt__(self, other):
        return self.gt(other)

    def __ge__(self, other):
        return self.gte(other)

    def __eq__(self, other):
        return self.eq(other)

    def __ne__(self, other):
        return self.ne(other)

    def __and__(self, other):
        return self.and_(other)

    def __or__(self, other):
        return self.or_(other)

    def __neg__(self):
        return self.neg_()

    def __abs__(self):
        return self.abs_()

    def __add__(self, other):
        return self.add(other)

    def __sub__(self, other):
        return self.sub(other)

    def __mul__(self, other):
        return self.mul(other)

    def __pow__(self, other):
        return self.pow(other)

    def __truediv__(self, other):
        return self.div(other)

    def apply(self, value: Any) -> Any:
        """
        Apply the expression to a given value. This is useful for evaluating the expression with specific inputs.

        Parameters
        ----------
        value
            The value to apply the expression to.

        Returns
        -------
        Any
            The result of applying the expression to the input value.
        """
        return self.__backend__().evaluate(value)

    def time(self) -> Expr:
        return Expr.from_rust(self.__backend__().time())

    def rolling(self, window: int) -> Expr:
        return Expr.from_rust(self.__backend__().rolling(window))

    def first(self) -> Expr:
        return Expr.from_rust(self.__backend__().first())

    def last(self) -> Expr:
        return Expr.from_rust(self.__backend__().last())

    def sum(self) -> Expr:
        return Expr.from_rust(self.__backend__().sum())

    def mean(self) -> Expr:
        return Expr.from_rust(self.__backend__().mean())

    def stddev(self) -> Expr:
        return Expr.from_rust(self.__backend__().stddev())

    def min(self) -> Expr:
        return Expr.from_rust(self.__backend__().min())

    def max(self) -> Expr:
        return Expr.from_rust(self.__backend__().max())

    def var(self) -> Expr:
        return Expr.from_rust(self.__backend__().var())

    def skew(self) -> Expr:
        return Expr.from_rust(self.__backend__().skew())

    def count(self) -> Expr:
        return Expr.from_rust(self.__backend__().count())

    def unique(self) -> Expr:
        return Expr.from_rust(self.__backend__().unique())

    def literal(self, value: float | int | str) -> Expr:
        return Expr.from_rust(PyExpr.literal(value))

    def debug(self) -> Expr:
        return Expr.from_rust(self.__backend__().debug())

    def slope(self) -> Expr:
        return Expr.from_rust(self.__backend__().slope())

    lt = _binary_op("lt")
    lte = _binary_op("lte")
    gt = _binary_op("gt")
    gte = _binary_op("gte")
    eq = _binary_op("eq", allow_str=True)
    ne = _binary_op("ne", allow_str=True)

    def and_(self, rhs: Expr) -> Expr:
        return Expr.from_rust(self.__backend__().and_(rhs.__backend__()))

    def or_(self, rhs: Expr) -> Expr:
        return Expr.from_rust(self.__backend__().or_(rhs.__backend__()))

    def not_(self) -> Expr:
        return Expr.from_rust(self.__backend__().not_())

    def neg_(self) -> Expr:
        return Expr.from_rust(self.__backend__().neg_())

    def abs_(self) -> Expr:
        return Expr.from_rust(self.__backend__().abs_())

    add = _binary_op("add_")
    sub = _binary_op("sub_")
    mul = _binary_op("mul_")
    div = _binary_op("div_")
    pow = _binary_op("pow")

    def clamp(self, min: Expr | float | int, max: Expr | float | int) -> Expr:
        return Expr.from_rust(self.__backend__().clamp_(_coerce(min), _coerce(max)))

    def when(self, condition: Expr) -> When:
        return When(condition=condition)

    def element(self) -> Expr:
        return Expr.from_rust(PyExpr.element())

    def every(self, interval: int) -> When:
        return When(condition=PyExpr.every(interval))

    def cast(self, to: RdDataType) -> Expr:
        return Expr.from_rust(self.__backend__().cast(str(to)))


def mean(metric_name: str) -> Expr:
    return select(metric_name)


def min(metric_name: str) -> Expr:
    return select(metric_name)


def max(metric_name: str) -> Expr:
    return select(metric_name)


def stddev(metric_name: str) -> Expr:
    return select(metric_name)


def select(metric: str) -> Expr:
    return Expr.from_rust(PyExpr.select(metric))


def when(condition: Expr) -> When:
    return When(condition=condition)


def lit(value: float | int | str) -> Expr:
    return Expr.from_rust(PyExpr.literal(value))


def element() -> Expr:
    return Expr.from_rust(PyExpr.element())


def every(interval: int) -> Every:
    return Every(interval=interval)


def generation() -> Expr:
    return Expr.from_rust(PyExpr.metric("index"))
