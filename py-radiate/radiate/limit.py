from radiate.radiate import PyLimit


class LimitBase:
    def __init__(self, limit: PyLimit):
        """
        Initialize the limit base with a Limit instance.
        :param limit: An instance of Limit.
        """
        self.limit = limit

    def __str__(self):
        """
        Return a string representation of the limit.
        :return: String representation of the limit.
        """
        return f"Limit(name={self.limit.name}, args={self.limit.args})"

    def __repr__(self):
        """
        Return a detailed string representation of the limit.
        :return: Detailed string representation of the limit.
        """
        return f"LimitBase(limit={self.limit})"


class SecondsLimit(LimitBase):
    """
    Limit the execution time of the engine.
    """

    def __init__(self, seconds: int):
        """
        Initialize the seconds limit.
        :param seconds: Number of seconds to limit the execution time.
        """
        super().__init__(limit=PyLimit.Seconds(seconds))


class ScoreLimit(LimitBase):
    """
    Limit the score of the engine.
    """

    def __init__(self, score: float):
        """
        Initialize the score limit.
        :param score: Score to limit the execution time.
        """
        super().__init__(limit=PyLimit.Score(score))


class GenerationsLimit(LimitBase):
    """
    Limit the number of generations of the engine.
    """

    def __init__(self, generations: int):
        """
        Initialize the generations limit.
        :param generations: Number of generations to limit the execution time.
        """
        super().__init__(limit=PyLimit.Generation(generations))
