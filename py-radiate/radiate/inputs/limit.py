from typing import Dict, Any, List
from .component import ComponentBase


class LimitBase(ComponentBase):
    def __init__(self, component: str, args: Dict[str, Any] = {}):
        super().__init__(component=component, args=args)

    def __str__(self):
        """
        Return a string representation of the limit.
        :return: String representation of the limit.
        """
        return f"Limit(name={self.component}, args={self.args})"

    def __repr__(self):
        """
        Return a detailed string representation of the limit.
        :return: Detailed string representation of the limit.
        """
        return self.__str__()


class SecondsLimit(LimitBase):
    """
    Limit the execution time of the engine.
    """

    def __init__(self, seconds: int):
        """
        Initialize the seconds limit.
        :param seconds: Number of seconds to limit the execution time.
        """
        if seconds <= 0:
            raise ValueError("Seconds limit must be a positive integer.")
        if not isinstance(seconds, int):
            raise TypeError("Seconds limit must be an integer.")
        super().__init__(component="seconds", args={"seconds": seconds})


class ScoreLimit(LimitBase):
    """
    Limit the score of the engine.
    """

    def __init__(self, score: float | List[float]):
        """
        Initialize the score limit.
        :param score: Score to limit the execution time.
        """
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
        super().__init__(component="score", args={"score": score})


class GenerationsLimit(LimitBase):
    """
    Limit the number of generations of the engine.
    """

    def __init__(self, generations: int):
        """
        Initialize the generations limit.
        :param generations: Number of generations to limit the execution time.
        """
        if generations <= 0:
            raise ValueError("Generations limit must be a positive integer.")
        if not isinstance(generations, int):
            raise TypeError("Generations limit must be an integer.")
        super().__init__(component="generations", args={"generations": generations})


class ConvergenceLimit(LimitBase):
    """
    Limit the convergence of the engine.
    """

    def __init__(self, window: int, epsilon: float):
        """
        Initialize the convergence limit.
        :param window: The number of generations to consider for convergence.
        :param epsilon: The threshold for convergence.
        """
        if window <= 0:
            raise ValueError("Window size must be a positive integer.")
        if not isinstance(window, int):
            raise TypeError("Window size must be an integer.")
        if epsilon < 0:
            raise ValueError("Epsilon must be a non-negative float.")
        if not isinstance(epsilon, (int, float)):
            raise TypeError("Epsilon must be a float or an integer.")
        super().__init__(
            component="convergence", args={"window": window, "epsilon": epsilon}
        )
