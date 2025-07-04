"""
This module provides common fixtures and utilities used across all test modules.
"""

from typing import List

import pytest
import random
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
        np.random.seed(seed)
    rd.random.set_seed(seed)
    return seed


@pytest.fixture
def small_int_population():
    """Create a small population of integer chromosomes for testing."""
    return [
        rd.Chromosome.int(length=5, value_range=(0, 10)),
        rd.Chromosome.int(length=5, value_range=(0, 10)),
        rd.Chromosome.int(length=5, value_range=(0, 10)),
        rd.Chromosome.int(length=5, value_range=(0, 10)),
        rd.Chromosome.int(length=5, value_range=(0, 10)),
    ]


@pytest.fixture
def small_float_population():
    """Create a small population of float chromosomes for testing."""
    return [
        rd.Chromosome.float(length=3, value_range=(-1.0, 1.0)),
        rd.Chromosome.float(length=3, value_range=(-1.0, 1.0)),
        rd.Chromosome.float(length=3, value_range=(-1.0, 1.0)),
        rd.Chromosome.float(length=3, value_range=(-1.0, 1.0)),
        rd.Chromosome.float(length=3, value_range=(-1.0, 1.0)),
    ]


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


class TestDataGenerator:
    """Utility class for generating test data."""

    @staticmethod
    def random_ints(length: int, min_val: int = 0, max_val: int = 100) -> List[int]:
        """Generate random integer list."""
        return [random.randint(min_val, max_val) for _ in range(length)]

    @staticmethod
    def random_floats(
        length: int, min_val: float = -1.0, max_val: float = 1.0
    ) -> List[float]:
        """Generate random float list."""
        return [random.uniform(min_val, max_val) for _ in range(length)]

    @staticmethod
    def random_strings(length: int, min_len: int = 3, max_len: int = 8) -> List[str]:
        """Generate random string list."""
        chars = "abcdefghijklmnopqrstuvwxyz"
        return [
            "".join(random.choices(chars, k=random.randint(min_len, max_len)))
            for _ in range(length)
        ]

    @staticmethod
    def random_bits(length: int) -> List[bool]:
        """Generate random boolean list."""
        return [random.choice([True, False]) for _ in range(length)]


@pytest.fixture
def data_generator():
    """Provide test data generator."""
    return TestDataGenerator()


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
