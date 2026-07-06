from .._rd import components
from .input import EngineInput, EngineInputType


class Filter(EngineInput):
    def __init__(self, component: str, **kwargs):
        super().__init__(
            component=component,
            input_type=EngineInputType.Filter,
            **kwargs,
        )

    @staticmethod
    def unique_score(threshold: float = 0.01, max_stagnation: int = 10):
        """
        The `UniqueScoreFilter` is a filter that removes individuals from the population that have a score
        that is too similar to other individuals. This can help to maintain diversity in the population and
        prevent premature convergence by ensuring that the population does not become dominated by a single
        solution.

        :param threshold: The minimum difference in score between two individuals for them to be considered unique.
        :param max_stagnation: The maximum number of generations an individual can remain in the population without being replaced.
        """
        return Filter(
            component=components.UNIQUE_SCORE_FILTER,
            max_stagnation=max_stagnation,
            threshold=threshold,
        )
