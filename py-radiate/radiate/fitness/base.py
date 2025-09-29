import abc
from radiate.radiate import PyFitnessFn


class FitnessBase[T](abc.ABC):
    """Base class for fitness functions in evolutionary algorithms."""

    def __init__(self, problem: PyFitnessFn):
        self.problem = problem
