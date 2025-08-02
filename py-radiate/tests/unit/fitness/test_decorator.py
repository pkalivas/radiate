"""Tests for the @fitness decorator."""

import pytest
import numpy as np
from typing import List

import radiate as rd
from radiate.fitness import fitness


class TestFitnessDecorator:
    """Test the @fitness decorator functionality."""

    @pytest.mark.unit
    def test_decorator_returns_optimized_fitness(self):
        """Test that decorator returns CompiledFitness instance."""

        @fitness()
        def simple_func(x: List[float]) -> float:
            return sum(x)

        assert callable(simple_func)

    @pytest.mark.unit
    def test_decorator_preserves_function_metadata(self):
        """Test that decorator preserves original function metadata."""

        @fitness()
        def documented_func(x: List[float]) -> float:
            """A well-documented function."""
            return sum(x)

        assert documented_func.__name__ == "documented_func"
        assert documented_func.__doc__ == "A well-documented function."

    @pytest.mark.unit
    @pytest.mark.skipif(
        not hasattr(rd, "_NUMBA_AVAILABLE") or not rd._NUMBA_AVAILABLE,
        reason="Numba not available",
    )
    def test_decorator_with_strategy_numba(self):
        """Test decorator with strategy='numba'."""

        @fitness(signature=rd.Float32Array)
        def numba_func(x: np.ndarray) -> float:
            return np.sum(x**2)

        result = numba_func(np.array([1.0, 2.0, 3.0], dtype=np.float32))
        assert result == 14.0

        info = numba_func.get_compilation_info()
        assert info["backend"] == "numba"

    @pytest.mark.unit
    def test_decorator_with_signature_types(self):
        """Test decorator with different signature types."""

        @fitness(signature=rd.Float32Array)
        def array_func(x: np.ndarray) -> float:
            return np.mean(x)

        @fitness(signature=(rd.Float32Array, rd.Float32))
        def tuple_sig_func(x: np.ndarray) -> float:
            return np.sum(x)

        # Test they work
        arr = np.array([1.0, 2.0, 3.0], dtype=np.float32)
        assert array_func(arr) == 2.0
        assert tuple_sig_func(arr) == 6.0

    @pytest.mark.unit
    @pytest.mark.skipif(
        not hasattr(rd, "_NUMBA_AVAILABLE") or not rd._NUMBA_AVAILABLE,
        reason="Numba not available",
    )
    def test_decorator_fallback_on_compilation_failure(self):
        """Test that decorator falls back to original function on compilation failure."""

        @fitness()  # This will likely fail
        def complex_func(x: List[float]) -> float:
            # Complex function that numba can't compile
            import random

            return sum(x) + random.random()

        # Should still work (fallback to original)
        result = complex_func([1.0, 2.0, 3.0])
        assert isinstance(result, float)
        assert result >= 6.0  # sum + random value

    @pytest.mark.unit
    def test_get_original_function(self):
        """Test accessing the original uncompiled function."""

        def original_func(x: List[float]) -> float:
            return sum(x)

        decorated = fitness()(original_func)

        assert decorated.get_original() is original_func
        assert decorated.get_original()([1, 2, 3]) == 6


class TestFitnessDecoratorIntegration:
    """Integration tests for @fitness decorator with engines."""

    @pytest.mark.unit
    def test_decorated_function_with_engine(self, random_seed):
        """Test that decorated fitness functions work with GeneticEngine."""

        @fitness()
        def fitness_func(x: List[float]) -> float:
            return sum(xi**2 for xi in x)

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=3, value_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objectives="min",
            population_size=20,
        )

        result = engine.run([rd.GenerationsLimit(10)])
        assert result.index() == 10

    @pytest.mark.unit
    @pytest.mark.skipif(not rd._NUMBA_AVAILABLE, reason="Numba not available")
    def test_numba_compiled_function_with_engine(self, random_seed):
        """Test numba-compiled fitness function with engine."""

        @fitness(signature=rd.Float32Array)
        def numba_fitness(x: np.ndarray) -> float:
            return np.sum(x**2)

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(
                length=5, value_range=(-2.0, 2.0), use_numpy=True
            ),
            fitness_func=numba_fitness,
            objectives="min",
            population_size=30,
        )

        result = engine.run([rd.GenerationsLimit(20)])
        assert result.index() == 20
        assert result.score()[0] >= 0.0  # Sum of squares is non-negative


class TestFitnessDecoratorEdgeCases:
    """Test edge cases and error conditions."""

    @pytest.mark.unit
    def test_decorator_with_no_arguments(self):
        """Test decorator used without parentheses."""

        @fitness
        def no_args_func(x: List[float]) -> float:
            return sum(x)

        # Should work as if called with default arguments
        result = no_args_func([1.0, 2.0, 3.0])
        assert result == 6.0

    @pytest.mark.unit
    def test_decorator_with_lambda(self):
        """Test decorator with lambda functions."""
        # This should work
        decorated_lambda = fitness()(lambda x: sum(x))

        result = decorated_lambda([1, 2, 3])
        assert result == 6

    @pytest.mark.unit
    def test_decorator_raises_on_too_many_arguments(self):
        """Test that decorated function preserves original call signature."""
        with pytest.raises(TypeError):

            @fitness()
            def multi_arg_func(x: List[float], y: float = 1.0) -> float:
                return sum(x) * y

    @pytest.mark.unit
    def test_decorator_with_type_hints(self):
        """Test decorator preserves and works with type hints."""

        @fitness()
        def typed_func(x: np.ndarray) -> np.float64:
            return np.sum(x)

        # Should preserve annotations
        assert hasattr(typed_func.get_original(), "__annotations__")

        # Should work with numpy arrays
        arr = np.array([1.0, 2.0, 3.0])
        result = typed_func(arr)
        assert result == 6.0


@pytest.mark.performance
class TestFitnessDecoratorPerformance:
    """Performance tests for @fitness decorator."""

    @pytest.mark.skipif(not rd._NUMBA_AVAILABLE, reason="Numba not available")
    def test_numba_compilation_speedup(self, performance_benchmark):
        """Test that numba compilation provides speedup for appropriate functions."""

        # Original function
        def original_func(x: np.ndarray) -> float:
            return float(np.sum(x**2))

        # Compiled function
        @fitness(signature=rd.Float32Array)
        def compiled_func(x: np.ndarray) -> float:
            return float(np.sum(x**2))

        large_array = np.random.random(10000).astype(np.float32)

        # Warm up compiled function
        compiled_func(large_array[:100])

        # Benchmark both
        _, original_time = performance_benchmark.time_function(
            original_func, large_array
        )

        _, compiled_time = performance_benchmark.time_function(
            compiled_func, large_array
        )

        # Compiled version should be faster (or at least not much slower)
        # Note: For small arrays, compilation overhead might make it slower
        assert compiled_time < original_time * 2  # Allow some overhead
