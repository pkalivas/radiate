"""
This module provides common fixtures and utilities used across all test modules.
"""

import os
import pytest
import random
import radiate as rd

try:
    import numpy as np

    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False
    np = None

# Enable test-only imports
os.environ["RADIATE_TESTING"] = "1"


@pytest.fixture(autouse=True)
def enable_test_imports():
    """Automatically enable test-only imports for all tests."""
    os.environ["RADIATE_TESTING"] = "1"
    yield


@pytest.fixture(scope="session")
def random_seed():
    """Set a consistent random seed for reproducible tests."""
    seed = 42
    random.seed(seed)
    if HAS_NUMPY:
        np.random.seed(seed)
    rd.random.seed(seed)
    return seed


@pytest.fixture
def xor_dataset():
    """Create XOR dataset for testing."""
    inputs = [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]
    outputs = [[0.0], [1.0], [1.0], [0.0]]
    return inputs, outputs


@pytest.fixture
def simple_regression_dataset():
    """Create a simple regression dataset for testing."""
    inputs = [[0.0], [1.0], [2.0], [3.0], [4.0]]
    outputs = [[0.0], [2.0], [4.0], [6.0], [8.0]]
    return inputs, outputs


@pytest.fixture
def graph_codec_simple():
    """Create a simple graph codec for testing."""
    return rd.GraphCodec.directed(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )


@pytest.fixture
def graph_simple():
    """Create a simple graph structure for testing."""
    codec = rd.GraphCodec.directed(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    return codec.decode(codec.encode())


@pytest.fixture
def tree_codec_simple():
    """Create a simple tree codec for testing."""
    return rd.TreeCodec(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sub()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        root=rd.Op.linear(),
    )


@pytest.fixture
def tree_simple():
    """Create a simple tree structure for testing."""
    codec = rd.TreeCodec(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sub()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        root=rd.Op.linear(),
    )
    return codec.decode(codec.encode())


@pytest.fixture
def simple_float_engine():
    """Create a simple float codec engine for testing."""
    codec = rd.FloatCodec.vector(length=10, init_range=(-1.0, 1.0))
    return rd.GeneticEngine(
        codec=codec,
        fitness_func=lambda x: sum(xi**2 for xi in x),
        objective="min",
        population_size=100,
        alters=[
            rd.UniformCrossover(0.5),
            rd.ArithmeticMutator(0.1),
        ],
    )


@pytest.fixture
def simple_multi_objective_engine():
    """Create a simple multi-objective float codec engine for testing."""
    codec = rd.FloatCodec.vector(length=10, init_range=(-1.0, 1.0))

    return rd.GeneticEngine(
        codec=codec,
        fitness_func=lambda x: [
            sum(xi**2 for xi in x),
            sum((xi - 0.5) ** 2 for xi in x),
        ],
        objective=["min", "min"],
        population_size=100,
        offspring_selector=rd.TournamentSelector(3),
        survivor_selector=rd.NSGA2Selector(),
        alters=[
            rd.UniformCrossover(0.5),
            rd.ArithmeticMutator(0.1),
        ],
    )


class PerformanceBenchmark:
    """Utility class for performance testing."""

    @staticmethod
    def time_function(func, *args, **kwargs):
        """Time a function execution."""
        import time

        start = time.time()
        result = func(*args, **kwargs)
        end = time.time()
        return result, end - start

    @staticmethod
    def memory_usage():
        """Get current memory usage (if psutil is available)."""
        try:
            import psutil

            process = psutil.Process()
            return process.memory_info().rss / 1024 / 1024  # MB
        except ImportError:
            return None


@pytest.fixture
def performance_benchmark():
    """Provide performance benchmarking utilities."""
    return PerformanceBenchmark()


# Test markers for different test types
def pytest_configure(config):
    """Configure custom pytest markers."""
    config.addinivalue_line("markers", "unit: mark test as a unit test")
    config.addinivalue_line("markers", "integration: mark test as an integration test")
    config.addinivalue_line("markers", "performance: mark test as a performance test")
    config.addinivalue_line("markers", "slow: mark test as slow running")
    config.addinivalue_line("markers", "regression: mark test as a regression test")
    config.addinivalue_line("markers", "smoke: mark test as a smoke test")
    config.addinivalue_line("markers", "skipif: mark test to skip if condition is true")
    config.addinivalue_line("markers", "bench: mark test as a benchmark test")
