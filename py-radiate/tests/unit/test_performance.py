import pytest
import gc

import radiate as rd


class TestCodecPerformance:
    """Performance tests for codecs."""

    @pytest.mark.performance
    def test_codec_encode_decode_speed(self, performance_benchmark):
        """Benchmark codec encode/decode operations."""
        codec = rd.FloatCodec.vector(length=100000, init_range=(-1.0, 1.0))

        def encode_decode_cycle():
            genotype = codec.encode()
            decoded = codec.decode(genotype)
            return decoded

        result, execution_time = performance_benchmark.time_function(
            encode_decode_cycle
        )

        assert len(result) == 100000
        assert execution_time < 0.2

    @pytest.mark.performance
    def test_large_matrix_codec_performance(self, performance_benchmark):
        """Benchmark large matrix codec operations."""
        codec = rd.IntCodec.matrix((100, 100), init_range=(0, 1000))

        def matrix_operations():
            genotype = codec.encode()
            decoded = codec.decode(genotype)
            return decoded

        result, execution_time = performance_benchmark.time_function(matrix_operations)

        assert len(result) == 100
        assert all(len(row) == 100 for row in result)
        assert execution_time < 0.1


class TestEnginePerformance:
    """Performance tests for the genetic engine."""

    @pytest.mark.performance
    def test_engine_small_problem_performance(self, performance_benchmark):
        """Benchmark engine performance on small problems."""

        def fitness_func(x: list[float]) -> float:
            return sum(xi**2 for xi in x)

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=10, init_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objective="min",
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.UniformCrossover(0.7), rd.ArithmeticMutator(0.1)],
        )

        def engine_run():
            return engine.run([rd.GenerationsLimit(50)])

        result, execution_time = performance_benchmark.time_function(engine_run)

        assert result.index() == 50
        assert execution_time < 5.0

    @pytest.mark.performance
    def test_engine_large_population_performance(self, performance_benchmark):
        """Benchmark engine performance with large population."""

        def fitness_func(x: list[float]) -> float:
            return sum(xi**2 for xi in x)

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=20, init_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objective="min",
            population_size=1000,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.UniformCrossover(0.7), rd.ArithmeticMutator(0.1)],
        )

        def engine_run():
            return engine.run([rd.GenerationsLimit(10)])

        result, execution_time = performance_benchmark.time_function(engine_run)

        assert result.index() == 10
        assert execution_time < 30.0


class TestMemoryPerformance:
    """Performance tests for memory usage."""

    @pytest.mark.performance
    def test_memory_usage_small_problem(self, performance_benchmark):
        """Test memory usage for small problems."""

        def fitness_func(x: list[float]) -> float:
            return sum(xi**2 for xi in x)

        initial_memory = performance_benchmark.memory_usage()

        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(length=10, init_range=(-1.0, 1.0)),
            fitness_func=fitness_func,
            objective="min",
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            survivor_selector=rd.EliteSelector(),
            alters=[rd.UniformCrossover(0.7), rd.ArithmeticMutator(0.1)],
        )

        engine.run([rd.GenerationsLimit(50)])

        final_memory = performance_benchmark.memory_usage()

        if initial_memory and final_memory:
            memory_increase = final_memory - initial_memory
            assert memory_increase < 50.0

    @pytest.mark.performance
    def test_memory_cleanup(self, performance_benchmark):
        """Test that memory is properly cleaned up after engine runs."""

        def fitness_func(x: list[float]) -> float:
            return sum(xi**2 for xi in x)

        initial_memory = performance_benchmark.memory_usage()

        # Run multiple engines
        for _ in range(5):
            engine = rd.GeneticEngine(
                codec=rd.FloatCodec.vector(length=50, init_range=(-1.0, 1.0)),
                fitness_func=fitness_func,
                objective="min",
                population_size=200,
                offspring_selector=rd.TournamentSelector(3),
                survivor_selector=rd.EliteSelector(),
                alters=[rd.UniformCrossover(0.7), rd.ArithmeticMutator(0.1)],
            )

            engine.run([rd.GenerationsLimit(10)])
            del engine  # Explicitly delete engine

        # Force garbage collection
        gc.collect()

        final_memory = performance_benchmark.memory_usage()

        if initial_memory and final_memory:
            memory_increase = final_memory - initial_memory
            assert memory_increase < 100.0  # Memory should be cleaned up


class TestScalabilityPerformance:
    """Performance tests for scalability."""

    @pytest.mark.performance
    def test_scalability_chromosome_length(self, performance_benchmark):
        """Test how performance scales with chromosome length."""
        lengths = [10, 50, 100, 500]
        execution_times = []

        def fitness_func(x: list[float]) -> float:
            return sum(xi**2 for xi in x)

        for length in lengths:
            engine = rd.GeneticEngine(
                codec=rd.FloatCodec.vector(length=length, init_range=(-1.0, 1.0)),
                fitness_func=fitness_func,
                objective="min",
                population_size=100,
                offspring_selector=rd.TournamentSelector(3),
                survivor_selector=rd.EliteSelector(),
                alters=[rd.UniformCrossover(0.7), rd.ArithmeticMutator(0.1)],
            )

            _, execution_time = performance_benchmark.time_function(
                engine.run, [rd.GenerationsLimit(20)]
            )
            execution_times.append(execution_time)

        # Performance should scale reasonably (not exponentially)
        for i in range(1, len(execution_times)):
            time_ratio = execution_times[i] / execution_times[i - 1]
            length_ratio = lengths[i] / lengths[i - 1]
            assert time_ratio < length_ratio * 2
