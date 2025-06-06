from typing import List
import radiate as rd


def __int_fitness_func(x: List[int]) -> int:
    """A simple fitness function for testing."""
    return sum(x)


def __int_codec() -> rd.IntCodec:
    """Create an IntCodec for testing."""
    return rd.IntCodec.vector(10, value_range=(0, 10))


def test_engine_set_population_size():
    pop_size = 123

    engine_one = rd.GeneticEngine(
        __int_codec(), __int_fitness_func, population_size=pop_size
    )

    engine_two = rd.GeneticEngine(__int_codec(), __int_fitness_func)
    engine_two.population_size(pop_size)

    assert engine_one.__dict__()["population_size"] == pop_size
    assert engine_two.__dict__()["population_size"] == pop_size


def test_engine_set_offspring_fraction():
    offspring_frac = 0.5

    engine_one = rd.GeneticEngine(
        __int_codec(), __int_fitness_func, offspring_fraction=offspring_frac
    )

    engine_two = rd.GeneticEngine(__int_codec(), __int_fitness_func)
    engine_two.offspring_fraction(offspring_frac)

    assert engine_one.__dict__()["offspring_fraction"] == offspring_frac
    assert engine_two.__dict__()["offspring_fraction"] == offspring_frac


def test_engine_set_ages():
    max_phenotype_age = 20
    max_species_age = 20
    species_threshold = 1.5

    engine_one = rd.GeneticEngine(
        __int_codec(),
        __int_fitness_func,
        max_phenotype_age=max_phenotype_age,
        max_species_age=max_species_age,
        species_threshold=species_threshold,
    )

    engine_two = rd.GeneticEngine(__int_codec(), __int_fitness_func)
    engine_two.max_age(
        max_phenotype_age=max_phenotype_age, max_species_age=max_species_age
    )
    engine_two.diversity(rd.HammingDistance(), species_threshold)

    assert engine_one.__dict__()["max_phenotype_age"] == max_phenotype_age
    assert engine_two.__dict__()["max_phenotype_age"] == max_phenotype_age
    assert engine_one.__dict__()["max_species_age"] == max_species_age
    assert engine_two.__dict__()["max_species_age"] == max_species_age
    assert engine_one.__dict__()["species_threshold"] == species_threshold
    assert engine_two.__dict__()["species_threshold"] == species_threshold


def test_engine_set_alters():
    alters = [rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.01)]

    engine_one = rd.GeneticEngine(__int_codec(), __int_fitness_func, alters=alters)

    engine_two = rd.GeneticEngine(__int_codec(), __int_fitness_func)
    engine_two.alters(alters)

    engine_one_alters = engine_one.__dict__()["alters"]
    engine_two_alters = engine_two.__dict__()["alters"]

    assert len(engine_one_alters) == len(alters)
    assert len(engine_two_alters) == len(alters)
    assert all(
        a1.__class__ == a2.__class__
        for a1, a2 in zip(engine_one_alters, engine_two_alters)
    )
    assert all(
        a1.args() == a2.alterer.args() for a1, a2 in zip(engine_one_alters, alters)
    )
    assert all(
        a1.args() == a2.alterer.args() for a1, a2 in zip(engine_two_alters, alters)
    )


def test_engine_set_selector():
    survivor_selector = rd.EliteSelector()
    offspring_selector = rd.BoltzmannSelector(4)

    engine_one = rd.GeneticEngine(
        __int_codec(),
        __int_fitness_func,
        survivor_selector=survivor_selector,
        offspring_selector=offspring_selector,
    )

    engine_two = rd.GeneticEngine(__int_codec(), __int_fitness_func)
    engine_two.survivor_selector(survivor_selector)
    engine_two.offspring_selector(offspring_selector)

    engine_one_survivor_selector = engine_one.__dict__()["survivor_selector"]
    engine_two_survivor_selector = engine_two.__dict__()["survivor_selector"]

    engine_one_offspring_selector = engine_one.__dict__()["offspring_selector"]
    engine_two_offspring_selector = engine_two.__dict__()["offspring_selector"]

    assert engine_one_survivor_selector.args() == survivor_selector.selector.args()
    assert engine_two_survivor_selector.args() == survivor_selector.selector.args()
    assert engine_one_offspring_selector.args() == offspring_selector.selector.args()
    assert engine_two_offspring_selector.args() == offspring_selector.selector.args()


def test_engine_can_minimize():
    num_genes = 5
    engine = rd.GeneticEngine(
        codec=rd.IntCodec.vector(num_genes, value_range=(0, 10)),
        fitness_func=lambda x: sum(x),
    )

    result = engine.run([rd.ScoreLimit(0), rd.GenerationsLimit(500)])

    assert result.value() == [0 for _ in range(num_genes)]
    assert result.score() == [0]
    assert result.index() < 500


def test_engine_can_maximize():
    target = "Hello, Radiate!"

    def fitness_func(x: List[str]) -> int:
        return sum(1 for i in range(len(target)) if x[i] == target[i])

    engine = rd.GeneticEngine(
        codec=rd.CharCodec.vector(len(target)),
        fitness_func=fitness_func,
        objectives="max",
        offspring_selector=rd.BoltzmannSelector(4),
    )

    result = engine.run([rd.ScoreLimit(len(target)), rd.GenerationsLimit(1000)])

    assert result.value() == list(target)
    assert result.score() == [len(target)]
    assert result.index() < 1000


def test_engine_minimizing_limits():
    import math

    A = 10.0
    RANGE = 5.12
    N_GENES = 2

    def fitness_fn(x: List[float]) -> float:
        value = A * N_GENES
        for i in range(N_GENES):
            value += x[i] ** 2 - A * math.cos((2.0 * 3.141592653589793 * x[i]))
        return value

    engine = rd.GeneticEngine(rd.FloatCodec.vector(2, (-RANGE, RANGE)), fitness_fn)
    engine.alters([rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.01)])

    result = engine.run([rd.ScoreLimit(0.0001), rd.GenerationsLimit(1000)])

    assert all(i < 0.001 for i in result.value())
    assert len(result.value()) == N_GENES
    assert result.index() < 1000
