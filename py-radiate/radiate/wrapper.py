from __future__ import annotations

from typing import Any
from abc import ABC


class PyObject[T](ABC):
    """
    Abstract base class for Python wrapper objects that wrap Rust objects.
    Provides common functionality for conversion between Python and Rust objects.
    """

    __slots__ = ["_pyobj"]

    def __init__(self):
        self._pyobj = None

    @classmethod
    def from_python(cls, py_obj: T):
        instance = cls.__new__(cls)
        instance._pyobj = py_obj
        return instance

    def to_python(self) -> T:
        return self._pyobj

    def __repr__(self) -> str:
        """Default representation using the inner object."""
        if "_pyobj" not in self.__dict__:
            return f"{self.__class__.__name__}"
        return f"{self.__class__.__name__}({self._pyobj})"

    def __eq__(self, other: Any) -> bool:
        """Compare with another wrapper or the inner object."""
        if isinstance(other, type(self)):
            return self.to_python() == other.to_python()
        return self.to_python() == other

    def __hash__(self) -> int:
        """Hash based on the inner object."""
        return hash(self.to_python())
