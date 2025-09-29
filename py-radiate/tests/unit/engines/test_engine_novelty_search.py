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


class TestEngineNoveltySearch:
    """Test suite for novelty search engine."""

    @pytest.mark.integration
    def test_engine_is_novel(self, random_seed):
        """Test engine with novelty search."""
        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(6, init_range=(-100.0, 100.0)),
            fitness_func=rd.NoveltySearch(
                descriptor=lambda x: x,
                distance=rd.CosineDistance(),
                k=15,
                threshold=0.03,
            ),
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            alters=[rd.UniformCrossover(0.5), rd.GaussianMutator(0.1)],
        )

        result = engine.run([rd.GenerationsLimit(100)])

        assert calc_population_diversity(result.population()) > 0.85, (
            "Population should have diversity"
        )
        assert result.index() == 100

    @pytest.mark.integration
    def test_int_engine_novelty_creates(self, random_seed):
        """Test engine with novelty search."""
        engine = rd.GeneticEngine(
            codec=rd.IntCodec.vector(6, init_range=(-100, 100)),
            fitness_func=rd.NoveltySearch(
                descriptor=lambda x: x,
                distance=rd.HammingDistance(),
                k=15,
                threshold=0.03,
            ),
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            alters=[rd.UniformCrossover(0.5), rd.ArithmeticMutator(0.1)],
        )

        result = engine.run([rd.GenerationsLimit(100)])

        assert calc_population_diversity(result.population()) > 0.85, (
            "Population should have diversity"
        )
        assert result.index() == 100
