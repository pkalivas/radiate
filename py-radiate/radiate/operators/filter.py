from .base import ComponentBase


class FilterBase(ComponentBase):
    """Base class for all filter components."""

    def __init__(self, component: str, **kwargs):
        super().__init__(component, **kwargs)


class UniqueScoreFilter(FilterBase):
    """Filter that removes individuals with duplicate scores."""

    def __init__(self, max_stagnation: int, threshold: float):
        super().__init__(
            component="UniqueScoreFilter",
            args={
                "max_stagnation": max_stagnation,
                "threshold": threshold,
            },
        )


class Filter:
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
        return UniqueScoreFilter(threshold=threshold, max_stagnation=max_stagnation)
