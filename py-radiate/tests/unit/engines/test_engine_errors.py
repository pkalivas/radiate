import radiate as rd
import pytest


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
        engine.run(rd.Limit.generations(10))


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
        engine.run(rd.Limit.generations(-1))  # Invalid limit


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
