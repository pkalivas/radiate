"""
This module provides common fixtures and utilities used across all test modules.
"""

import random

import pytest

import radiate as rd

try:
    import numpy as np

    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False
    np = None


@pytest.fixture(scope="session")
def random_seed():
    """Set a consistent random seed for reproducible tests."""
    seed = 42
    random.seed(seed)
    if HAS_NUMPY:
        np.random.seed(seed)  # type: ignore
    rd.random.seed(seed)
    return seed


"""
Data sets
"""


@pytest.fixture
def xor_dataset():
    """Create XOR dataset for testing."""
    return rd.DataSet.xor().pair()


@pytest.fixture
def squared_dataset():
    """Create a simple regression dataset for testing."""
    inputs = [[0.0], [1.0], [2.0], [3.0], [4.0]]
    outputs = [[0.0], [2.0], [4.0], [6.0], [8.0]]
    return inputs, outputs


@pytest.fixture
def memory_dataset():
    """Create a dataset for testing memory-based fitness functions."""
    return rd.DataSet.simple_memory(7).pair()


@pytest.fixture
def cubic_dataset():
    return rd.DataSet.cubic().pair()


"""
Engine Fixtures
"""


@pytest.fixture
def graph_1x1_engine():
    """Create a simple 1x1 graph engine for testing."""
    return rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sum()],
        edge=rd.Op.weight(),
        output=rd.Op.sum(),
    ).alter(
        rd.Cross.graph(0.05),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1),
    )


@pytest.fixture
def tree_2x1_engine():
    """Create a simple 2x1 tree engine for testing."""
    return rd.Engine.tree(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sub()],
        leaf=[rd.Op.var(0), rd.Op.var(1)],
        root=rd.Op.sum(),
    ).alter(
        rd.Cross.tree(0.05),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.hoist(0.1),
    )


@pytest.fixture
def simple_float_engine():
    """Create a simple float codec engine for testing."""
    return (
        rd.Engine.float(10, init_range=(-1.0, 1.0))
        .fitness(lambda x: sum(xi**2 for xi in x))
        .minimizing()
        .size(100)
        .alter(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.1))
    )


@pytest.fixture
def simple_multi_objective_engine():
    """Create a simple multi-objective float codec engine for testing."""
    return (
        rd.Engine.float(10, init_range=(-1.0, 1.0))
        .fitness(
            lambda x: [
                sum(xi**2 for xi in x),
                sum((xi - 0.5) ** 2 for xi in x),
            ]
        )
        .objective(rd.MIN, rd.MIN)
        .size(100)
        .select(rd.Select.tournament(3), rd.Select.nsga2())
        .alter(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.1))
    )


@pytest.fixture
def simple_bit_20_bit_engine():
    """Create a simple bit codec engine for testing."""
    return (
        rd.Engine.bit(20).size(150).alter(rd.Cross.uniform(0.5), rd.Mutate.uniform(0.1))
    )


"""
Metric Fixtures
"""


@pytest.fixture
def simple_metric_set():
    """Create a simple MetricSet for testing."""
    return rd.MetricSet(
        one=list(range(10)),
        two=list(range(10, 20)),
        const=[5.0] * 10,
    )


@pytest.fixture
def large_metric_set():
    """Create a larger MetricSet for testing."""
    return rd.MetricSet(
        one=list(range(100)),
        two=list(range(100, 200)),
        three=list(range(200, 300)),
        four=list(range(300, 400)),
        five=list(range(400, 500)),
    )


"""
Performance Benchmarking Utilities
"""


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
            import psutil  # type: ignore

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
