# Testing Guide for Radiate

This document describes the comprehensive testing strategy for the Radiate Python package.

## Overview

Radiate uses a multi-layered testing approach to ensure code quality, reliability, and performance:

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test how components work together
- **Performance Tests**: Benchmark performance and detect regressions
- **Smoke Tests**: Quick tests for basic functionality
- **Regression Tests**: Prevent performance degradation

## Test Structure

```
tests/
├── conftest.py              # Pytest configuration and fixtures
├── unit/                    # Unit tests
│   ├── test_codecs.py       # Codec functionality tests
│   └── test_operators.py    # Genetic operator tests
├── integration/             # Integration tests
│   └── test_engine_integration.py  # Engine integration tests
├── performance/             # Performance tests
│   └── test_performance.py  # Benchmark tests
└── __init__.py
```

## Test Categories

### Unit Tests (`@pytest.mark.unit`)

Unit tests focus on individual components and functions:

- **Codec Tests**: Test encoding/decoding functionality
- **Operator Tests**: Test genetic operators (crossover, mutation, selection)
- **Edge Cases**: Test error handling and boundary conditions

**Example:**
```python
@pytest.mark.unit
def test_int_codec_vector_creation():
    """Test creating an integer codec for vectors."""
    codec = IntCodec.vector(length=5, value_range=(0, 10))
    genotype = codec.encode()
    
    assert len(genotype) == 1
    assert len(genotype[0]) == 5
    assert all(0 <= gene <= 10 for gene in genotype[0])
```

### Integration Tests (`@pytest.mark.integration`)

Integration tests verify that multiple components work together:

- **Engine Tests**: Test complete genetic algorithm runs
- **Multi-Objective Tests**: Test multi-objective optimization
- **Error Handling**: Test system-level error handling

**Example:**
```python
@pytest.mark.integration
def test_engine_int_minimization(random_seed):
    """Test engine with integer codec for minimization."""
    def fitness_func(x: List[int]) -> float:
        return sum(x)
    
    engine = GeneticEngine(
        codec=IntCodec.vector(length=5, value_range=(0, 10)),
        fitness_func=fitness_func,
        objective="min"
    )
    
    result = engine.run([ScoreLimit(0), GenerationsLimit(100)])
    
    assert result.value() == [0] * 5
    assert result.score()[0] == 0.0
```

### Performance Tests (`@pytest.mark.performance`)

Performance tests benchmark execution time and memory usage:

- **Speed Tests**: Measure execution time for various operations
- **Memory Tests**: Monitor memory usage and cleanup
- **Scalability Tests**: Test performance scaling with problem size
- **Regression Tests**: Prevent performance degradation

**Example:**
```python
@pytest.mark.performance
def test_engine_small_problem_performance(benchmark):
    """Benchmark engine performance on small problems."""
    def fitness_func(x: List[float]) -> float:
        return sum(xi**2 for xi in x)
    
    engine = GeneticEngine(
        codec=FloatCodec.vector(length=10, value_range=(-1.0, 1.0)),
        fitness_func=fitness_func,
        objective="min"
    )
    
    result, execution_time = benchmark.time_function(
        engine.run, [GenerationsLimit(50)]
    )
    
    assert execution_time < 5.0  # Should complete within 5 seconds
```

### Smoke Tests (`@pytest.mark.smoke`)

Smoke tests provide quick verification of basic functionality:

- **Basic Operations**: Test fundamental features work
- **Import Tests**: Verify modules can be imported
- **Simple Examples**: Test basic usage patterns

### Regression Tests (`@pytest.mark.regression`)

Regression tests prevent performance and functionality degradation:

- **Performance Regression**: Ensure performance doesn't worsen
- **Functionality Regression**: Ensure features continue to work
- **Memory Regression**: Ensure memory usage doesn't increase

## Running Tests

### Prerequisites

Install test dependencies:

```bash
pip install -e ".[dev]"
pip install pytest pytest-cov pytest-benchmark pytest-timeout pytest-xdist
```

### Using the Test Runner

The `run_tests.py` script provides convenient test execution:

```bash
# Run all tests
python run_tests.py --all

# Run specific test types
python run_tests.py --unit
python run_tests.py --integration
python run_tests.py --performance

# Run with coverage
python run_tests.py --unit --coverage

# Run smoke tests (default)
python run_tests.py

# Install test dependencies
python run_tests.py --install-deps
```

### Using pytest Directly

Run specific test categories:

```bash
# Unit tests
pytest tests/unit/ -m unit

# Integration tests
pytest tests/integration/ -m integration

# Performance tests
pytest tests/performance/ -m performance

# Smoke tests
pytest tests/ -m smoke

# All tests except slow ones
pytest tests/ -m "not slow"

# With coverage
pytest tests/ --cov=radiate --cov-report=html
```

### Test Markers

Use pytest markers to run specific test types:

- `@pytest.mark.unit`: Unit tests
- `@pytest.mark.integration`: Integration tests
- `@pytest.mark.performance`: Performance tests
- `@pytest.mark.slow`: Tests that take longer to run
- `@pytest.mark.regression`: Regression tests
- `@pytest.mark.smoke`: Smoke tests

## Test Fixtures

Common fixtures are defined in `conftest.py`:

- `random_seed`: Sets consistent random seed for reproducible tests
- `small_int_population`: Small population of integer chromosomes
- `small_float_population`: Small population of float chromosomes
- `xor_dataset`: XOR problem dataset
- `simple_regression_dataset`: Simple regression dataset
- `graph_codec_simple`: Simple graph codec
- `tree_codec_simple`: Simple tree codec
- `data_generator`: Test data generator utility
- `benchmark`: Performance benchmarking utility

## Continuous Integration

GitHub Actions automatically runs tests on:

- **Push to main/develop**: Full test suite
- **Pull requests**: Unit, integration, and smoke tests
- **Performance tests**: Run on main branch pushes
- **Security checks**: Run on main branch pushes

### CI Jobs

1. **Test Matrix**: Runs on multiple OS and Python versions
2. **Performance**: Benchmarks performance on main branch
3. **Security**: Security checks on main branch
4. **Build**: Package building on multiple platforms

## Writing Tests

### Unit Test Guidelines

1. **Test one thing**: Each test should verify one specific behavior
2. **Use descriptive names**: Test names should clearly describe what's being tested
3. **Arrange-Act-Assert**: Structure tests with clear sections
4. **Test edge cases**: Include tests for boundary conditions and error cases
5. **Use fixtures**: Leverage shared fixtures for common setup

**Example:**
```python
@pytest.mark.unit
def test_float_codec_bounds():
    """Test float codec respects bounds."""
    # Arrange
    codec = FloatCodec.vector(length=10, value_range=(-5, 5))
    
    # Act
    genotype = codec.encode()
    
    # Assert
    for gene in genotype[0]:
        assert -5 <= gene <= 5
```

### Integration Test Guidelines

1. **Test real scenarios**: Use realistic problem sizes and configurations
2. **Verify end-to-end**: Test complete workflows
3. **Check results**: Verify that the algorithm produces expected results
4. **Test error handling**: Ensure graceful handling of errors

### Performance Test Guidelines

1. **Use benchmarks**: Use pytest-benchmark for accurate timing
2. **Set thresholds**: Define performance expectations
3. **Test scalability**: Verify performance scales reasonably
4. **Monitor memory**: Check memory usage and cleanup

## Test Data

### Synthetic Data

Use the `TestDataGenerator` class for generating test data:

```python
@pytest.fixture
def data_generator():
    return TestDataGenerator()

def test_with_synthetic_data(data_generator):
    ints = data_generator.random_ints(100, 0, 100)
    floats = data_generator.random_floats(50, -1.0, 1.0)
    strings = data_generator.random_strings(10, 3, 8)
    bits = data_generator.random_bits(20)
```

### Real Datasets

Common test datasets are available as fixtures:

- `xor_dataset`: XOR classification problem
- `simple_regression_dataset`: Linear regression problem

## Coverage

Generate coverage reports:

```bash
# Generate HTML coverage report
pytest tests/ --cov=radiate --cov-report=html

# Generate XML coverage report (for CI)
pytest tests/ --cov=radiate --cov-report=xml

# View coverage in terminal
pytest tests/ --cov=radiate --cov-report=term-missing
```

Coverage reports are available in the `htmlcov/` directory.

## Debugging Tests

### Verbose Output

```bash
pytest tests/ -v
```x

### Debug Mode

```bash
pytest tests/ --pdb
```

### Test Discovery

```bash
# List all tests
pytest --collect-only

# List tests matching pattern
pytest --collect-only -k "codec"
```

## Best Practices

1. **Keep tests fast**: Unit tests should run quickly
2. **Use meaningful assertions**: Assertions should clearly indicate what went wrong
3. **Test both success and failure cases**: Don't just test happy paths
4. **Use parameterized tests**: For testing multiple similar cases
5. **Mock external dependencies**: Don't rely on external services in unit tests
6. **Clean up resources**: Ensure tests don't leave side effects
7. **Document complex tests**: Add comments for non-obvious test logic

## Troubleshooting

### Common Issues

1. **Import errors**: Ensure the package is installed in development mode
2. **Rust compilation errors**: Ensure Rust toolchain is properly installed
3. **Performance test failures**: Check if system is under load
4. **Memory test failures**: Check for memory leaks in the code

### Getting Help

- Check the test output for detailed error messages
- Use `pytest -v` for verbose output
- Use `pytest --tb=long` for full tracebacks
- Check the CI logs for environment-specific issues 