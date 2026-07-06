import pytest

import radiate as rd


@pytest.mark.unit
def test_engine_empty_population():
    """Test engine handles empty population gracefully."""

    def fitness_func(x: list[int]) -> float:
        return sum(x)

    with pytest.raises(ValueError):
        engine = rd.Engine(
            codec=rd.IntCodec(shape=3, init_range=(0, 10)),
            fitness_func=fitness_func,
            objective=rd.MIN,
            population_size=0,  # Invalid
        )
        engine.limit(rd.Limit.generations(10)).run()


@pytest.mark.unit
def test_engine_invalid_limits():
    """Test engine handles invalid limits gracefully."""

    def fitness_func(x: list[int]) -> float:
        return sum(x)

    engine = rd.Engine(
        codec=rd.IntCodec(shape=3, init_range=(0, 10)),
        fitness_func=fitness_func,
        objective=rd.MIN,
    )

    with pytest.raises(ValueError):
        engine.limit(rd.Limit.generations(-1)).run()  # Invalid limit


@pytest.mark.unit
def test_engine_invalid_numeric_alters():
    """
    Test that the engine fails when invalid alters are applied to a typed engine
    and handles them gracefully.
    """

    def fitness_func(x: list[list[bool]]) -> float:
        return sum(sum(row) for row in x)

    engine = (
        rd.Engine.bit(shape=[20, 20])
        # Bit Engine's can't use numeric alters, so this should cause an error
        .alters(rd.Cross.blend(), rd.Mutate.gaussian())
        .fitness(fitness_func)
        .limit(rd.Limit.generations(10))
    )

    with pytest.raises(ValueError):
        _ = engine.run()


@pytest.mark.unit
def test_engine_invalid_alters(graph_1x1_engine):
    """Test that the engine fails when invalid alters are applied and handles them gracefully."""

    def fit(_: rd.Graph) -> float:
        return 0.0  # doesn't matter - this test should fail on engine setup, not during evaluation

    engine = (
        graph_1x1_engine.fitness(fit)
        .limit(rd.Limit.generations(10))
        .alters(rd.Cross.uniform(0.5), rd.Mutate.gaussian())
    )  # Both these alters are invalid for a graph codec, so this should cause an error

    with pytest.raises(ValueError):
        _ = engine.run()
