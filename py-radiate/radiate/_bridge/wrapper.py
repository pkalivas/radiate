from __future__ import annotations

from collections.abc import Callable
from typing import Any, TYPE_CHECKING
from abc import ABC

if TYPE_CHECKING:
    from radiate._typing import RdDataType


class RsObject(ABC):
    """
    Abstract base class for Python wrapper objects that wrap Rust objects.
    Provides common functionality for conversion between Python and Rust objects.
    """

    __slots__ = ["_pyobj", "_cache", "_dtype"]
    _pyobj: Any
    _cache: dict[str, Any]

    def __init__(self, pyobj: Any = None):
        self._pyobj = pyobj
        self._cache = {}
        self._dtype = None

    @classmethod
    def from_rust(cls, py_obj: Any | dict):
        instance = cls.__new__(cls)
        if isinstance(py_obj, dict):
            instance.__dict__.update(py_obj)  # type: ignore
        else:
            instance._pyobj = py_obj
        instance._cache = {}
        instance._dtype = None
        return instance

    def dtype(self) -> "RdDataType":
        """
        Get the data type of the underlying Rust object, if applicable.
        :return: The data type as a RdDataType.
        """
        from ..dtype import Null

        if self._dtype is not None:
            return self._dtype
        else:
            backend = self.__backend__()
            if backend is not None:
                self._dtype = backend.dtype() if hasattr(backend, "dtype") else Null()
                return self._dtype
            else:
                return Null()

    def __backend__(self) -> Any:
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

    def try_invalidate_cache(self, key: str) -> None:
        """
        Invalidate a cached value by key.
        :param key: The cache key to invalidate.
        """
        if key in self._cache:
            del self._cache[key]

    def try_get_cache(self, key: str, acquire_fn: Callable[[], Any]) -> Any:
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
