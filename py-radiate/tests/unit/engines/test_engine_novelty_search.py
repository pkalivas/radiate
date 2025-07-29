from typing import List
import radiate as rd
import pytest


class TestEngineNoveltySearch:
    """Test suite for novelty search engine."""

    @pytest.mark.integration
    def test_engine_is_novel(self, random_seed):
        """Test engine with novelty search."""
        engine = rd.GeneticEngine(
            codec=rd.FloatCodec.vector(6, value_range=(-100.0, 100.0)),
            fitness_func=rd.NoveltySearch(
                distance=rd.CosineDistance(),
                k=15,
                threshold=0.03,
            ),
            population_size=100,
            offspring_selector=rd.TournamentSelector(3),
            alters=[rd.UniformCrossover(0.5), rd.GaussianMutator(0.1)],
        )

        result = engine.run([rd.GenerationsLimit(100)])

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
                max(desc[d] for desc in descriptors)
                - min(desc[d] for desc in descriptors)
                for d in range(dimension)
            )
            return total_range / (dimension * 200.0)

        assert calc_population_diversity(result.population()) > 0.9, (
            "Population should have diversity"
        )
        assert result.index() == 100

    @pytest.mark.integration
    def test_graph_engine_novelty_creates(self, random_seed):
        """Test engine with novelty search."""

        def descriptor(individual: rd.Graph) -> List[float]:
            """Extract descriptor from individual."""
            return [1, 2, 3]  # Simple fixed descriptor for testing

        codec = rd.GraphCodec.directed(
            shape=(1, 1),
            vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
            edge=rd.Op.weight(),
            output=rd.Op.linear(),
        )

        valid_engine = rd.GeneticEngine(
            codec=codec,
            fitness_func=rd.NoveltySearch(
                distance=rd.CosineDistance(),
                descriptor=descriptor,
                k=15,
                threshold=0.03,
            ),
            alters=[
                rd.GraphCrossover(0.5, 0.5),
                rd.GraphMutator(0.1, 0.1),
            ],
        )

        valid_engine.run([rd.GenerationsLimit(3)])

        fail_engine = rd.GeneticEngine(
            codec=codec,
            fitness_func=rd.NoveltySearch(
                distance=rd.CosineDistance(),
                k=15,
                threshold=0.03,
            ),
            alters=[
                rd.GraphCrossover(0.5, 0.5),
                rd.GraphMutator(0.1, 0.1),
            ],
        )

        with pytest.raises(TypeError):
            fail_engine.run([rd.GenerationsLimit(3)])
