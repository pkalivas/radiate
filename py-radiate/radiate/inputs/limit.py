from typing import Dict, Any
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

    def __init__(self, score: float):
        """
        Initialize the score limit.
        :param score: Score to limit the execution time.
        """
        if score < 0:
            raise ValueError("Score limit must be a non-negative float.")
        if not isinstance(score, (int, float)):
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
