import functools
import inspect
from typing import Any, Callable, Optional, Tuple, Union

from radiate.datatypes.classes import DataType
from radiate.dependancies import (
    _NUMBA_AVAILABLE,
    _NUMPY_AVAILABLE,
    numba,
    np,
)


def fitness[T](
    func: Callable[[T], Any] = None,
    *,
    signature: Union[Tuple[DataType, DataType] | DataType | None] = None,
):
    """
    Decorator to automatically optimize fitness functions.

    Args:
        signature: Type signature for compilation optimization

    Example:
        @fitness(signature=rd.Float32Array)
        def my_fitness(x: np.ndarray) -> float:
            return np.sum(x**2)
    """

    def decorator(func: Callable) -> CompiledFitness:
        return CompiledFitness(func, signature)

    if func is None:
        return decorator
    else:
        return decorator(func)


class CompiledFitness:
    """Wrapper for optimized fitness functions with automatic compilation."""

    def __init__(
        self,
        func: Callable,
        signature: Union[Tuple[DataType, DataType] | DataType | None] = None,
    ):
        self.original_func = func
        self.compiler = FitnessCompiler(func, signature)
        self.compiled_func = self.compiler.compile()

        # Copy function metadata
        functools.update_wrapper(self, func)

    def __call__(self, input):
        """Call the optimized fitness function."""
        return self.compiled_func(input)

    def get_compilation_info(self) -> dict:
        """Get information about the compilation process."""
        return self.compiler.compile_info

    def get_original(self) -> Callable:
        """Get the original uncompiled function."""
        return self.original_func


class FitnessCompiler:
    """Handles compilation and optimization of fitness functions."""

    def __init__(
        self,
        func: Callable,
        signature: Union[Tuple[DataType, DataType] | DataType | None] = None,
    ):
        self.original_func = func
        self.signature = signature
        self.compile_info = {
            "backend": "original",
            "input_type": None,
            "output_type": None,
        }

    def compile(self) -> Callable:
        """Compile using the specified or best available strategy."""
        input_type, output_type = self._parse_fitness_signature()

        self.compile_info["input_type"] = input_type
        self.compile_info["output_type"] = output_type

        if _NUMPY_AVAILABLE:
            if input_type is np.ndarray:
                numba_compiled = self._try_numba()
                if numba_compiled:
                    self.compile_info["backend"] = "numba"
                    return numba_compiled

        return self.original_func

    def _parse_fitness_signature(self) -> Tuple[Any, Any]:
        sig = inspect.signature(self.original_func)

        return_type = sig.return_annotation
        input_type = list(
            map(
                lambda p: p.annotation,
                filter(
                    lambda p: p.kind == inspect.Parameter.POSITIONAL_OR_KEYWORD,
                    sig.parameters.values(),
                ),
            )
        )

        if len(input_type) != 1:
            raise TypeError(
                f"Fitness function must have exactly one input parameter, found {len(input_type)}"
            )

        return next(iter(input_type)), return_type

    def _try_numba(self) -> Optional[Callable]:
        """Attempt numba compilation."""
        if not _NUMBA_AVAILABLE:
            return None

        if self.signature:
            numba_sig = self._convert_dtype_to_numba(self.signature)
            return (numba.jit(numba_sig) if numba_sig else numba.jit())(
                self.original_func
            )

        return numba.jit()(self.original_func)

    def _convert_dtype_to_numba(self, dtype_sig):
        """Convert DataType signature to numba types."""
        if not _NUMBA_AVAILABLE:
            return None

        type_map, py_type_map = self._get_type_mappings()

        if isinstance(dtype_sig, tuple):
            return self._handle_tuple_signature(dtype_sig, type_map)
        elif dtype_sig in type_map:
            return self._handle_single_signature(dtype_sig, type_map, py_type_map)

        return None

    def _get_type_mappings(self):
        """Get type mappings for numba conversion."""
        from radiate.datatypes.classes import (
            Bool,
            BoolArray,
            Float32,
            Float32Array,
            Float32Matrix,
            Float64,
            Float64Array,
            Float64Matrix,
            Int32,
            Int32Array,
            Int32Matrix,
            Int64,
            Int64Array,
            Int64Matrix,
        )

        type_map = {
            Int32: numba.int32,
            Int64: numba.int64,
            Float32: numba.float32,
            Float64: numba.float64,
            Bool: numba.boolean,
            Int32Array: numba.int32[:],
            Int64Array: numba.int64[:],
            Float32Array: numba.float32[:],
            Float64Array: numba.float64[:],
            BoolArray: numba.boolean[:],
            Int32Matrix: numba.int32[:, :],
            Int64Matrix: numba.int64[:, :],
            Float32Matrix: numba.float32[:, :],
            Float64Matrix: numba.float64[:, :],
        }

        py_type_map = {
            "int": numba.int64,
            "float": numba.float64,
            "bool": numba.boolean,
            "List[int]": numba.int32[:],
            "List[float]": numba.float32[:],
            "List[bool]": numba.boolean[:],
        }

        return type_map, py_type_map

    def _handle_tuple_signature(self, dtype_sig, type_map):
        """Handle tuple signature format: (input_type, output_type)."""
        input_type = type_map.get(dtype_sig[0])
        output_type = type_map.get(dtype_sig[1])

        if input_type and output_type:
            return output_type(input_type)
        return None

    def _handle_single_signature(self, dtype_sig, type_map, py_type_map):
        """Handle single type signature with return type inference."""
        source = inspect.getsource(self.original_func)

        if "->" not in source:
            return type_map.get(dtype_sig)

        # Extract return type annotation
        return_annotation = source.split("->")[1].strip()
        if ":" in return_annotation:
            output_type = return_annotation.split(":")[0].strip().lower()
        else:
            output_type = return_annotation.strip().lower()

        if output_type in py_type_map:
            return py_type_map[output_type](type_map[dtype_sig])

        return type_map.get(dtype_sig)
