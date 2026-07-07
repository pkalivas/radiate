from radiate.radiate import PyFitnessFn

from .._bridge import RsObject


class FitnessBase[T](RsObject):
    """Base class for fitness functions in evolutionary algorithms."""

    def __init__(self, problem: PyFitnessFn):
        self._pyobj = problem
