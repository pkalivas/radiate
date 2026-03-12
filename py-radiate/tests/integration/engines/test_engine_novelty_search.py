import radiate as rd
import pytest


def calc_population_diversity(population: rd.Population[float]) -> float:
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
        rd.Engine.float(6, init_range=(-100.0, 100.0))
        .fitness(
            rd.NoveltySearch(
                descriptor=lambda x: x,
                distance=rd.Dist.cosine(),
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

    @rd.novelty(distance=rd.Dist.hamming(), k=15, threshold=0.03)
    def novelty(phenotype: list[int]) -> list[int]:
        return phenotype

    engine = (
        rd.Engine.int(6, init_range=(-100, 100))
        .fitness(novelty)
        .size(100)
        .select(offspring=rd.Select.tournament(3))
        .alters(rd.Cross.uniform(0.5), rd.Mutate.arithmetic(0.1))
    )

    result = engine.run(rd.GenerationsLimit(100))

    assert calc_population_diversity(result.population()) > 0.85, (  # type: ignore
        "Population should have diversity"
    )
    assert result.index() == 100
