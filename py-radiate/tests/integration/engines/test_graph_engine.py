import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_graph_xor(xor_dataset, random_seed):
    """Test engine with graph codec for XOR problem."""
    inputs, outputs = xor_dataset

    engine = rd.GeneticEngine(
        codec=rd.GraphCodec.directed(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
            edge=rd.Op.weight(),
            output=rd.Op.sigmoid(),
        ),
        fitness_func=rd.Regression(inputs, outputs),
        objective="min",
        population_size=100,
        offspring_selector=rd.BoltzmannSelector(4.0),
        alters=[
            rd.GraphCrossover(0.5, 0.5),
            rd.OperationMutator(0.07, 0.05),
            rd.GraphMutator(0.1, 0.1, False),
        ],
    )

    result = engine.run([rd.ScoreLimit(0.1), rd.GenerationsLimit(1000)])

    assert result.score()[0] < 0.1
    assert result.index() <= 1000
    assert isinstance(result.value(), rd.Graph)


@pytest.mark.integration
def test_engine_graph_regression_with_speciation(
    simple_regression_dataset, random_seed
):
    """Test engine with graph codec and speciation for regression."""
    inputs, outputs = simple_regression_dataset

    codec = rd.GraphCodec.directed(
        shape=(1, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=rd.Regression(inputs, outputs),
        objective="min",
        population_size=100,
        species_threshold=0.1,
        diversity=rd.NeatDistance(excess=0.1, disjoint=0.1, weight_diff=0.5),
        alters=[
            rd.GraphCrossover(0.5, 0.5),
            rd.OperationMutator(0.07, 0.05),
            rd.GraphMutator(0.1, 0.1),
        ],
    )

    result = engine.run([rd.ScoreLimit(0.1), rd.GenerationsLimit(500)])

    # Testing in multithreaded mode can lead to slightly different results so we
    # relax the assertion a bit by allowing a few # of species
    assert len(result.species()) in [2, 3, 4], "Should maintain multiple species"
    assert result.index() <= 500
    assert isinstance(result.value(), rd.Graph)
