from __future__ import annotations

import sys
from importlib import import_module
from importlib.util import find_spec
from types import ModuleType
from typing import TYPE_CHECKING, Any

if hasattr(sys, "_is_gil_enabled"):
    _GIL_ENABLED = sys._is_gil_enabled()
else:
    _GIL_ENABLED = True


class _LazyModule(ModuleType):
    """Lazy-loading proxy module for optional dependencies."""

    def __init__(self, module_name: str, *, module_available: bool):
        self._module_available = module_available
        self._module_name = module_name
        self._globals = globals()
        super().__init__(module_name)

    def _import(self) -> ModuleType:
        module = import_module(self.__name__)
        self._globals[self._module_name] = module
        self.__dict__.update(module.__dict__)
        return module

    def __getattr__(self, name: str) -> Any:
        if self._module_available:
            module = self._import()
            return getattr(module, name)
        else:
            msg = f"'{name}' requires {self._module_name!r} to be installed. "
            msg += f"Install with: pip install 'radiate[{self._module_name}]'"
            raise ModuleNotFoundError(msg) from None


def _lazy_import(module_name: str) -> tuple[ModuleType, bool]:
    """Lazy import with availability checking."""
    if module_name in sys.modules:
        return sys.modules[module_name], True

    try:
        module_spec = find_spec(module_name)
        module_available = not (module_spec is None or module_spec.loader is None)
    except ModuleNotFoundError:
        module_available = False

    return (
        _LazyModule(module_name=module_name, module_available=module_available),
        module_available,
    )


_PANDAS_AVAILABLE = False
_POLARS_AVAILABLE = False


if TYPE_CHECKING:
    try:
        import numpy as np
        _NUMPY_AVAILABLE = True
    except ModuleNotFoundError:
        _NUMPY_AVAILABLE = False
else:
    np, _NUMPY_AVAILABLE = _lazy_import("numpy")
    pl, _POLARS_AVAILABLE = _lazy_import("polars")
    pd, _PANDAS_AVAILABLE = _lazy_import("pandas")
