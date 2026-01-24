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


@pytest.mark.integration
def test_engine_graph_with_recurrent_connections(memory_dataset, random_seed):
    """Test engine with graph codec and elitism for classification."""
    inputs, outputs = memory_dataset

    codec = rd.GraphCodec.recurrent(
        shape=(1, 1),
        vertex=rd.Op.all_ops(),
        edge=[rd.Op.weight(), rd.Op.identity()],
        output=rd.Op.sigmoid(),
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=rd.Regression(inputs, outputs),
        objective="min",
        population_size=250,
        alters=[
            rd.GraphCrossover(0.5, 0.5),
            rd.OperationMutator(0.1, 0.05),
            rd.GraphMutator(0.05, 0.05),
        ],
    )

    result = engine.run([rd.ScoreLimit(0.001), rd.GenerationsLimit(500)])

    assert result.score()[0] < 0.001
    assert result.index() <= 500
    assert isinstance(result.value(), rd.Graph)
    assert len(result.population()) == 250

    graph = result.value()

    for _ in range(25):
        for inp, out in zip(inputs, outputs):
            pred = graph.eval([inp])[0][0]
            expected = out[0]
            assert abs(pred - expected) < 0.1

        # Reset the graph state between evaluations
        graph.reset()


@pytest.mark.integration
def test_engine_graph_recurrent_class_acc(memory_dataset, random_seed):
    """Test engine with graph codec and elitism for classification."""
    inputs, outputs = memory_dataset
    ohe = []
    for out in outputs:
        if out[0] == 0.0:
            ohe.append([1.0, 0.0])
        else:
            ohe.append([0.0, 1.0])

    outputs = ohe

    codec = rd.GraphCodec.gru(
        shape=(1, 2),
        vertex=rd.Op.all_ops(),
        edge=[rd.Op.weight(), rd.Op.identity()],
        output=rd.Op.sigmoid(),
    )

    engine = rd.GeneticEngine(
        codec=codec,
        fitness_func=rd.Regression(inputs, outputs, loss="cross_entropy"),
        objective="min",
        alters=[
            rd.GraphCrossover(0.5, 0.5),
            rd.OperationMutator(0.1, 0.05),
            rd.GraphMutator(0.05, 0.05),
        ],
    )

    result = engine.run([rd.ScoreLimit(0.01), rd.GenerationsLimit(500)])

    assert result.score()[0] < 0.01
    assert result.index() <= 500
    assert isinstance(result.value(), rd.Graph)

    acc = rd.calc_accuracy(result.value(), inputs, outputs, loss="cross_entropy")

    assert acc.sample_count() == len(inputs)
    assert acc.recall() > 0.99
    assert acc.loss() < 0.01
