import radiate as rd
import pytest


def calc_population_diversity(population: rd.Population) -> float:
    """Calculate diversity of the population."""
    descriptors = [
        [g.allele() for chrom in individual.genotype() for g in chrom]
        for individual in population
    ]
    if not descriptors:
        return 0.0

    dimension = len(descriptors[0])
    total_range = sum(
        max(desc[d] for desc in descriptors) - min(desc[d] for desc in descriptors)
        for d in range(dimension)
    )
    return total_range / (dimension * 200.0)


@pytest.mark.integration
def test_engine_is_novel(random_seed):
    """Test engine with novelty search."""
    engine = (
        rd.Engine.float(6, (-100.0, 100.0))
        .fitness(
            rd.NoveltySearch(
                descriptor=lambda x: x,
                distance=rd.CosineDistance(),
                k=15,
                threshold=0.03,
            )
        )
        .size(100)
        .select(rd.Select.tournament(3))
        .alters(rd.Cross.uniform(0.5), rd.Mutate.gaussian(0.1))
    )

    result = engine.run(rd.GenerationsLimit(100))

    assert calc_population_diversity(result.population()) > 0.85, (
        "Population should have diversity"
    )
    assert result.index() == 100


@pytest.mark.integration
def test_int_engine_novelty_with_decorator_creates(random_seed):
    """Test engine with novelty search."""

    @rd.novelty(distance=rd.HammingDistance(), k=15, threshold=0.03)
    def descriptor(phenotype: list[int]) -> list[int]:
        return phenotype

    engine = (
        rd.Engine(rd.IntCodec(6, (-100, 100)), descriptor)
        .size(100)
        .select(offspring=rd.TournamentSelector(3))
        .alters(rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.1))
    )

    result = engine.run(rd.GenerationsLimit(100))

    assert calc_population_diversity(result.population()) > 0.85, (
        "Population should have diversity"
    )
    assert result.index() == 100
