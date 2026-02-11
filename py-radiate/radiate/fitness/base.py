import abc
from radiate.radiate import PyFitnessFn
from radiate.wrapper import RsObject


class FitnessBase[T](RsObject[PyFitnessFn]):
    """Base class for fitness functions in evolutionary algorithms."""

    def __init__(self, problem: PyFitnessFn):

        self._pyobj = problem
