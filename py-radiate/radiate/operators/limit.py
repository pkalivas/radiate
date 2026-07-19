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


# class Limit:
#     @classmethod
#     def score2(cls, value: float) -> ScoreLimit:
#         return ScoreLimit(value)

#     @staticmethod
#     def score(value: float) -> ScoreLimit:
#         return ScoreLimit(value)

#     @staticmethod
#     def generations(n: int) -> GenerationsLimit:
#         return GenerationsLimit(n)

#     @staticmethod
#     def seconds(secs: int) -> SecondsLimit:
#         return SecondsLimit(secs)

#     @staticmethod
#     def convergence(window: int, threshold: float) -> ConvergenceLimit:
#         return ConvergenceLimit(window, threshold)

#     @staticmethod
#     def metric(
#         name: str = "count.evaluation", limit=lambda metric: metric.sum() > 1000
#     ) -> MetricLimit:
#         return MetricLimit(name, limit)

#     @staticmethod
#     def expr(expr: Expr) -> ExprLimit:
#         return ExprLimit(expr)


# class LimitBase(ComponentBase):
#     def __init__(self, component: str, args: Dict[str, Any] = {}):
#         super().__init__(component=component, args=args)

#     def __str__(self):
#         """
#         Return a string representation of the limit.
#         :return: String representation of the limit.
#         """
#         return f"Limit(name={self.component}, args={self.args})"

#     def __repr__(self):
#         """
#         Return a detailed string representation of the limit.
#         :return: Detailed string representation of the limit.
#         """
#         return self.__str__()


# class SecondsLimit(LimitBase):
#     """
#     Limit the execution time of the engine.
#     """

#     def __init__(self, seconds: int):
#         """
#         Initialize the seconds limit.
#         :param seconds: Number of seconds to limit the execution time.
#         """
#         if seconds <= 0:
#             raise ValueError("Seconds limit must be a positive integer.")
#         if not isinstance(seconds, int):
#             raise TypeError("Seconds limit must be an integer.")
#         super().__init__(component="seconds", args={"seconds": seconds})


# class ScoreLimit(LimitBase):
#     """
#     Limit the score of the engine.
#     """

#     def __init__(self, score: float | List[float]):
#         """
#         Initialize the score limit.
#         :param score: Score to limit the execution time.
#         """
#         if isinstance(score, list):
#             if not all(isinstance(s, (int, float)) for s in score):
#                 raise TypeError("Score limit must be a list of floats or integers.")
#             score = [float(s) for s in score]
#         else:
#             if not isinstance(score, (int, float)):
#                 raise TypeError("Score limit must be a float or an integer.")
#             score = [float(score)]
#         if any(s < 0 for s in score):
#             raise ValueError("Score limit must be a non-negative float.")
#         if not all(isinstance(s, (int, float)) for s in score):
#             raise TypeError("Score limit must be a float or an integer.")
#         super().__init__(component="score", args={"score": score})


# class GenerationsLimit(LimitBase):
#     """
#     Limit the number of generations of the engine.
#     """

#     def __init__(self, generations: int):
#         """
#         Initialize the generations limit.
#         :param generations: Number of generations to limit the execution time.
#         """
#         if generations <= 0:
#             raise ValueError("Generations limit must be a positive integer.")
#         if not isinstance(generations, int):
#             raise TypeError("Generations limit must be an integer.")
#         super().__init__(component="generations", args={"generations": generations})


# class ConvergenceLimit(LimitBase):
#     """
#     Limit the convergence of the engine.
#     """

#     def __init__(self, window: int, epsilon: float):
#         """
#         Initialize the convergence limit.
#         :param window: The number of generations to consider for convergence.
#         :param epsilon: The threshold for convergence.
#         """
#         if window <= 0:
#             raise ValueError("Window size must be a positive integer.")
#         if not isinstance(window, int):
#             raise TypeError("Window size must be an integer.")
#         if epsilon < 0:
#             raise ValueError("Epsilon must be a non-negative float.")
#         if not isinstance(epsilon, (int, float)):
#             raise TypeError("Epsilon must be a float or an integer.")
#         super().__init__(
#             component="convergence", args={"window": window, "epsilon": epsilon}
#         )


# class MetricLimit(LimitBase):
#     """
#     Limit the metric of the engine.
#     """

#     def __init__(self, name: str, limit: Callable[[Metric], bool]):
#         """
#         Initialize the metric limit.
#         :param name: The name of the metric to limit.
#         :param limit: A callable that takes a Metric and returns a bool.
#         """
#         if not isinstance(name, str):
#             raise TypeError("Metric name must be a string.")
#         if not callable(limit):
#             raise TypeError(
#                 "Metric limit must be a callable that takes a Metric and returns a bool."
#             )

#         def _wrap(metric):
#             from ..engine.metrics import Metric as _Metric

#             return limit(_Metric.from_rust(metric))

#         super().__init__(
#             component="metric",
#             args={"name": name, "limit": _wrap},
#         )


# class ExprLimit(LimitBase):
#     """
#     Limit the engine based on a custom expression (Expr).
#     """

#     def __init__(self, expr: Expr):
#         """
#         Initialize the expression limit.
#         :param expr: An Expr that evaluates to a boolean value to determine if the limit is reached.
#         """
#         if not isinstance(expr, Expr):
#             raise TypeError("Expr limit must be an instance of Expr.")
#         super().__init__(component="expr", args={"expr": expr.__backend__()})
