from __future__ import annotations

from .._rd import components
from ..dsl.expr import Expr
from .input import EngineInput, EngineInputType


class Limit(EngineInput):
    def __init__(self, component: str, **kwargs):
        super().__init__(
            component=component, input_type=EngineInputType.Limit, **kwargs
        )

    @staticmethod
    def score(score: float | list[float]) -> Limit:
        if isinstance(score, list):
            if not all(isinstance(s, (int, float)) for s in score):
                raise TypeError("Score limit must be a list of floats or integers.")
            score = [float(s) for s in score]
        else:
            if not isinstance(score, (int, float)):
                raise TypeError("Score limit must be a float or an integer.")
            score = [float(score)]
        if any(s < 0 for s in score):
            raise ValueError("Score limit must be a non-negative float.")
        if not all(isinstance(s, (int, float)) for s in score):
            raise TypeError("Score limit must be a float or an integer.")
        return Limit(components.SCORE_LIMIT, score=score)

    @staticmethod
    def generations(n: int) -> Limit:
        if n <= 0:
            raise ValueError("Generations limit must be a positive integer.")
        if not isinstance(n, int):
            raise TypeError("Generations limit must be an integer.")
        return Limit(components.GENERATIONS_LIMIT, generations=n)

    @staticmethod
    def seconds(secs: int) -> Limit:
        if secs <= 0:
            raise ValueError("Seconds limit must be a positive integer.")
        if not isinstance(secs, int):
            raise TypeError("Seconds limit must be an integer.")
        return Limit(components.SECONDS_LIMIT, seconds=secs)

    @staticmethod
    def convergence(window: int, threshold: float) -> Limit:
        if window <= 0:
            raise ValueError("Window size must be a positive integer.")
        if not isinstance(window, int):
            raise TypeError("Window size must be an integer.")
        if threshold < 0:
            raise ValueError("Threshold must be a non-negative float.")
        if not isinstance(threshold, (int, float)):
            raise TypeError("Threshold must be a float or an integer.")
        return Limit(components.CONVERGENCE_LIMIT, window=window, epsilon=threshold)

    @staticmethod
    def metric(
        name: str = "count.evaluation", limit=lambda metric: metric.sum() > 1000
    ) -> Limit:
        def _wrap(metric):
            from ..engine.metrics import Metric as _Metric

            return limit(_Metric.from_rust(metric))

        return Limit(components.METRIC_LIMIT, name=name, limit=_wrap)

    @staticmethod
    def expr(expr: Expr) -> Limit:
        return Limit(components.EXPR_LIMIT, expr=expr)
