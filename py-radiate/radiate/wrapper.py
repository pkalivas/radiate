from __future__ import annotations

from typing import Any
from abc import ABC


class PyObject[T](ABC):
    """
    Abstract base class for Python wrapper objects that wrap Rust objects.
    Provides common functionality for conversion between Python and Rust objects.
    """

    __slots__ = ["_pyobj", "_cache"]
    _pyobj: T
    _cache: dict[str, Any]

    def __init__(self, pyobj: T | None = None):
        self._pyobj = pyobj
        self._cache = {}

    @classmethod
    def from_rust(cls, py_obj: T | dict):
        instance = cls.__new__(cls)
        if isinstance(py_obj, dict):
            instance.__dict__.update(py_obj)
        else:
            instance._pyobj = py_obj
        instance._cache = {}
        return instance

    def __backend__(self) -> T:
        return self._pyobj

    def __repr__(self) -> str:
        """Default representation using the inner object."""
        if "_pyobj" not in self.__dict__:
            return f"{self.__class__.__name__}"
        return f"{self.__class__.__name__}({self._pyobj})"

    def __eq__(self, other: Any) -> bool:
        """Compare with another wrapper or the inner object."""
        if isinstance(other, type(self)):
            return self.__backend__() == other.__backend__()
        return self.__backend__() == other

    def __hash__(self) -> int:
        """Hash based on the inner object."""
        return hash(self.__backend__())

    def try_get_cache(self, key: str, acquire_fn: callable) -> Any:
        """
        Try to get a cached value by key, if not present, acquire it using the provided function.
        :param key: The cache key.
        :param acquire_fn: A callable that produces the value if not cached.
        :return: The cached or newly acquired value.
        """
        if key in self._cache:
            return self._cache[key]
        value = acquire_fn()
        self._cache[key] = value
        return value
