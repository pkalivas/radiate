from typing import Dict
from .param import EngineParam

class Limit(EngineParam):

    def __init__(self, name: str, args: Dict[str, any] = None):
        """
        Initialize the limit.
        :param name: Name of the limit.
        :param args: Arguments for the limit.
        """
        super().__init__(name=name, args=args)


class SecondsLimit(Limit):
    """
    Limit the execution time of the engine.
    """
    def __init__(self, seconds: int):
        """
        Initialize the seconds limit.
        :param seconds: Number of seconds to limit the execution time.
        """
        super().__init__(
            name='seconds',
            args={'seconds': str(seconds)}
        )

class ScoreLimit(Limit):
    """
    Limit the score of the engine.
    """
    def __init__(self, score: float):
        """
        Initialize the score limit.
        :param score: Score to limit the execution time.
        """
        super().__init__(
            name='score',
            args={'score': str(score)}
        )

class GenerationsLimit(Limit):
    """
    Limit the number of generations of the engine.
    """
    def __init__(self, generations: int):
        """
        Initialize the generations limit.
        :param generations: Number of generations to limit the execution time.
        """
        super().__init__(
            name='generations',
            args={'generations': str(generations)}
        )