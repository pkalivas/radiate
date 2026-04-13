from __future__ import annotations
from typing import Any

from radiate._bridge.input import RsObject
from radiate.radiate import PyExpr


class Then(RsObject):
    def __init__(self, condition: Expr | int, then_expr: Expr):
        self.condition = condition
        self.then_expr = then_expr

    def __repr__(self) -> str:
        return f"Then(condition={self.condition}, then={self.then_expr})"

    def otherwise(self, else_expr: Expr | float | int | str) -> Expr:
        else_expr = (
            else_expr
            if isinstance(else_expr, Expr)
            else Expr.from_rust(PyExpr.literal(else_expr))
        )

        if isinstance(self.condition, int):
            return Expr.from_rust(
                PyExpr.every(
                    self.condition,
                    self.then_expr.__backend__(),
                    else_expr.__backend__(),
                )
            )
        else:
            return Expr.from_rust(
                PyExpr.when_then_otherwise(
                    condition=self.condition.__backend__(),
                    then_expr=self.then_expr.__backend__(),
                    else_expr=else_expr.__backend__(),
                )
            )


class When(RsObject):
    def __init__(self, condition: Expr):
        self.condition = condition

    def __repr__(self) -> str:
        return f"When(condition={self.condition})"

    def then(self, then_expr: Expr | float | int | str) -> Then:
        then_expr = (
            then_expr
            if isinstance(then_expr, Expr)
            else Expr.from_rust(PyExpr.literal(then_expr))
        )

        return Then(condition=self.condition, then_expr=then_expr)


class Every(RsObject):
    def __init__(self, interval: int):
        self.interval = interval

    def __repr__(self) -> str:
        return f"Every(interval={self.interval})"

    def then(self, then_expr: Expr | float | int | str) -> Then:
        then_expr = (
            then_expr
            if isinstance(then_expr, Expr)
            else Expr.from_rust(PyExpr.literal(then_expr))
        )

        return Then(condition=self.interval, then_expr=then_expr)


class Expr(RsObject):
    """
    Base class for all expressions in the Radiate DSL. This class serves as a wrapper around Rust expressions, allowing them to be used seamlessly in Python.

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

    def lt(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().lt(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().lt(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for comparison")

    def lte(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().lte(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().lte(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for comparison")

    def gt(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().gt(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().gt(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for comparison")

    def gte(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().gte(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().gte(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for comparison")

    def eq(self, rhs: Expr | float | int | str) -> Expr:
        if isinstance(rhs, (float, int, str)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().eq(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().eq(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for comparison")

    def ne(self, rhs: Expr | float | int | str) -> Expr:
        if isinstance(rhs, (float, int, str)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().ne(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().ne(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for comparison")

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

    def add(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().add_(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().add_(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for addition")

    def sub(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().sub_(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().sub_(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for subtraction")

    def mul(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().mul_(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().mul_(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for multiplication")

    def div(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().div_(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().div_(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for division")

    def pow(self, rhs: Expr | float | int) -> Expr:
        if isinstance(rhs, (float, int)):
            rhs_expr = PyExpr.literal(rhs)
            return Expr.from_rust(self.__backend__().pow_(rhs_expr))
        elif isinstance(rhs, Expr):
            return Expr.from_rust(self.__backend__().pow_(rhs.__backend__()))
        else:
            raise TypeError("Unsupported type for exponentiation")

    def clamp(self, min: Expr | float | int, max: Expr | float | int) -> Expr:
        if isinstance(min, (float, int)):
            min_expr = PyExpr.literal(min)
        elif isinstance(min, Expr):
            min_expr = min.__backend__()
        else:
            raise TypeError("Unsupported type for clamp min")

        if isinstance(max, (float, int)):
            max_expr = PyExpr.literal(max)
        elif isinstance(max, Expr):
            max_expr = max.__backend__()
        else:
            raise TypeError("Unsupported type for clamp max")

        return Expr.from_rust(self.__backend__().clamp_(min_expr, max_expr))

    def when(self, condition: Expr) -> When:
        return When(condition=condition)

    def element(self) -> Expr:
        return Expr.from_rust(PyExpr.element())

    def every(self, interval: int) -> When:
        return When(condition=PyExpr.every(interval))


def mean(metric_name: str) -> Expr:
    return metric(metric_name)


def min(metric_name: str) -> Expr:
    return metric(metric_name)


def max(metric_name: str) -> Expr:
    return metric(metric_name)


def stddev(metric_name: str) -> Expr:
    return metric(metric_name)


def metric(metric: str) -> Expr:
    return Expr.from_rust(PyExpr.metric(metric))


def when(condition: Expr) -> When:
    return When(condition=condition)


def lit(value: float | int | str) -> Expr:
    return Expr.from_rust(PyExpr.literal(value))


def element() -> Expr:
    return Expr.from_rust(PyExpr.element())


def every(interval: int) -> Every:
    return Every(interval=interval)
