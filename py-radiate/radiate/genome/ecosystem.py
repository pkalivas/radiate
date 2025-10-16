from __future__ import annotations

from radiate.genome.population import Population
from radiate.genome.species import Species
from radiate.wrapper import PyObject
from radiate.radiate import PyEcosystem


class Ecosystem[T](PyObject[PyEcosystem]):
    def __init__(self, inner: PyEcosystem):
        super().__init__()

        if isinstance(inner, PyEcosystem):
            self._pyobj = inner
        else:
            raise TypeError(f"Expected PyEcosystem, got {type(inner)}")

    def __repr__(self):
        return self.__backend__().__repr__()

    def population(self) -> Population[T]:
        return Population.from_rust(self.__backend__().population)

    def species(self) -> list[Species[T]]:
        return [Species.from_rust(s) for s in self.__backend__().species]
