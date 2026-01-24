import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_permutation_tsp(random_seed):
    """Test engine with permutation codec for TSP-like problem."""

    # Simple TSP-like fitness: minimize sum of adjacent differences
    def fitness_func(x: list[int]) -> float:
        return sum(abs(x[i] - x[i - 1]) for i in range(1, len(x)))

    engine = rd.GeneticEngine(
        codec=rd.PermutationCodec([0, 1, 2, 3, 4]),
        fitness_func=fitness_func,
        objective="min",
        population_size=50,
        offspring_selector=rd.TournamentSelector(3),
        survivor_selector=rd.EliteSelector(),
        alters=[rd.PartiallyMappedCrossover(0.7), rd.InversionMutator(0.1)],
    )

    result = engine.run([rd.GenerationsLimit(100)])

    assert result.index() <= 100
    assert len(set(result.value())) == 5
    assert all(0 <= x < 5 for x in result.value())
