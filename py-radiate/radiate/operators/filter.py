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
