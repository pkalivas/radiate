from __future__ import annotations

from abc import ABC
from collections.abc import Callable
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from ._typing import RdDataType


class RsObject(ABC):
    """
    Abstract base class for Python wrapper objects that wrap Rust objects.
    Provides common functionality for conversion between Python and Rust objects.
    """

    __slots__ = ["_pyobj", "_cache", "_dtype"]
    _pyobj: Any
    _cache: dict[str, Any]

    def __new__(cls, *args, **kwargs):
        instance = super().__new__(cls)
        instance._pyobj = None
        instance._cache = {}
        instance._dtype = None
        return instance

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
        from .dsl.dtype import Null

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
        if "_pyobj" not in self.__dict__:
            return f"{self.__class__.__name__}"
        return f"{self.__class__.__name__}({self._pyobj})"

    def __eq__(self, other: Any) -> bool:
        if isinstance(other, type(self)):
            return self.__backend__() == other.__backend__()
        return self.__backend__() == other

    def __hash__(self) -> int:
        return hash(self.__backend__())

    def try_invalidate_cache(self, key: str) -> None:
        if key in self._cache:
            del self._cache[key]

    def try_get_cache(self, key: str, acquire_fn: Callable[[], Any]) -> Any:
        if key in self._cache:
            return self._cache[key]
        value = acquire_fn()
        self._cache[key] = value
        return value


class LazyRsObject(RsObject):
    """
    A subclass of RsObject that allows for lazy initialization of the underlying Rust object.
    """

    def __init__(self, pyobj: Any = None):
        super().__init__(pyobj)
        self._initialized = False

    def _initialize(self) -> Callable[[], Any]:
        """
        Method to initialize the underlying Rust object. Should be overridden by subclasses.
        """
        raise NotImplementedError("Subclasses must implement the _initialize method.")

    def __backend__(self) -> Any:
        if not self._initialized:
            build_fn = self._initialize()
            self._pyobj = build_fn()
            self._initialized = True
        return super().__backend__()
