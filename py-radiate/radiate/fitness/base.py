from radiate.radiate import PyFitnessFn
from radiate._bridge.wrapper import RsObject


class FitnessBase(RsObject):
    """Base class for fitness functions in evolutionary algorithms."""

    def __init__(self, problem: PyFitnessFn):
        self._pyobj = problem
