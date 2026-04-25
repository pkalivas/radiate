from __future__ import annotations
from typing import TYPE_CHECKING

from radiate._bridge.wrapper import RsObject
from radiate.radiate import PyRate

if TYPE_CHECKING:
    from radiate.expr import Expr


class Rate(RsObject):
    CYCLICAL = "cyclical"
    FIXED = "fixed"
    LINEAR = "linear"
    STEPWISE = "stepwise"
    EXPONENTIAL = "exponential"
    EXPR = "expr"

    def __init__(
        self,
        rate_type: str,
        start: float = 0.0,
        end: float = 1.0,
        duration: int = 1,
        shape: str = "sine",
        steps: list[tuple[int, float]] | None = None,
        expr: Expr | None = None,
    ):
        if rate_type == self.FIXED:
            self._pyobj = PyRate.fixed(start)
        elif rate_type == self.LINEAR:
            self._pyobj = PyRate.linear(start, end, duration)
        elif rate_type == self.CYCLICAL:
            self._pyobj = PyRate.cyclical(start, end, duration, shape)
        elif rate_type == self.STEPWISE:
            if steps is None:
                raise ValueError("Steps must be provided for step rate type.")
            self._pyobj = PyRate.stepwise(steps)
        elif rate_type == self.EXPONENTIAL:
            self._pyobj = PyRate.exponential(start, end, duration)
        elif rate_type == self.EXPR:
            self._pyobj = PyRate.expression(expr)
        else:
            raise ValueError(f"Unknown rate type: {rate_type}")

    def value(self, index: int) -> float:
        """
        Get the rate value at a specific index.

        :param index: The index to get the rate value for.
        :return: The rate value at the specified index.
        """
        return self._pyobj.value(index)

    @staticmethod
    def fixed(rate: float):
        return Rate(Rate.FIXED, rate)

    @staticmethod
    def linear(start: float, end: float, duration: int):
        return Rate(Rate.LINEAR, start, end, duration)

    @staticmethod
    def sine(min: float, max: float, periods: int):
        return Rate(Rate.CYCLICAL, min, max, periods, "sine")

    @staticmethod
    def triangular(min: float, max: float, periods: int):
        return Rate(Rate.CYCLICAL, min, max, periods, "triangular")

    @staticmethod
    def stepwise(steps: list[tuple[int, float]]):
        return Rate(Rate.STEPWISE, steps=steps)

    @staticmethod
    def exp(start: float, end: float, half_life: int):
        return Rate(Rate.EXPONENTIAL, start, end, half_life)

    @staticmethod
    def expr(expr: Expr):
        return Rate(Rate.EXPR, expr=expr.__backend__())


def fixed(rate: float):
    return Rate(Rate.FIXED, rate)


def sine(min: float, max: float, periods: int):
    return Rate(Rate.CYCLICAL, min, max, periods, "sine")


def triangular(min: float, max: float, periods: int):
    return Rate(Rate.CYCLICAL, min, max, periods, "triangular")


def linear(start: float, end: float, duration: int):
    return Rate(Rate.LINEAR, start, end, duration)


def stepwise(steps: list[tuple[int, float]]):
    return Rate(Rate.STEPWISE, steps=steps)


def exp(start: float, end: float, half_life: int):
    return Rate(Rate.EXPONENTIAL, start, end, half_life)
