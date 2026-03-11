import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_graph_xor(xor_dataset, random_seed):
    """Test engine with graph codec for XOR problem."""
    inputs, outputs = xor_dataset

    engine = (
        rd.Engine.graph(
            shape=(2, 1),
            vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
            edge=rd.Op.weight(),
            output=rd.Op.sigmoid(),
        )
        .regression(inputs, outputs)
        .select(offspring=rd.Select.boltzmann(4.0))
        .alters(
            rd.Cross.graph(0.5, 0.5),
            rd.Mutate.op(0.07, 0.05),
            rd.Mutate.graph(0.1, 0.1, False),
        )
        .limit(rd.Limit.score(0.1), rd.Limit.generations(1000))
    )

    result = engine.run()

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

    engine = (
        rd.Engine(codec)
        .regression(inputs, outputs)
        .diversity(rd.Dist.neat(0.1, 0.1, 0.5), 0.1)
        .alters(
            rd.Cross.graph(0.5, 0.5),
            rd.Mutate.op(0.07, 0.05),
            rd.Mutate.graph(0.1, 0.1),
        )
    )

    result = engine.run(rd.Limit.score(0.1), rd.Limit.generations(500))

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
        vertex=rd.Op.default_vertex_ops(),
        edge=[rd.Op.weight(), rd.Op.identity()],
        output=rd.Op.sigmoid(),
    )

    engine = (
        rd.Engine(codec)
        .size(250)
        .regression(inputs, outputs)
        .limit(rd.Limit.score(0.01), rd.Limit.generations(2000))
        .alters(
            rd.Cross.graph(0.5, 0.5),
            rd.Mutate.op(0.1, 0.05),
            rd.Mutate.graph(0.05, 0.05),
        )
    )

    result = engine.run()

    assert result.index() <= 2000
    assert isinstance(result.value(), rd.Graph)
    assert len(result.population()) == 250

    graph = result.value()

    for _ in range(25):
        for inp, out in zip(inputs, outputs):
            pred = graph.eval(inp)[0]
            expected = out[0]
            assert abs(pred - expected) < 0.5, f"Expected {expected}, got {pred}"

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

    engine = (
        rd.Engine(codec)
        .regression(inputs, outputs, loss=rd.XEnt)
        .alters(
            rd.Cross.graph(0.5, 0.5),
            rd.Mutate.op(0.1, 0.05),
            rd.Mutate.graph(0.05, 0.05),
        )
    )

    result = engine.run(rd.Limit.score(0.01), rd.Limit.generations(500))

    assert result.score()[0] < 0.01
    assert result.index() <= 500
    assert isinstance(result.value(), rd.Graph)

    acc = rd.accuracy(result.value(), inputs, outputs, loss=rd.XEnt)

    assert acc.sample_count() == len(inputs)
    assert acc.recall() is not None and acc.recall() > 0.99  # type: ignore
    assert acc.loss() is not None and acc.loss() < 0.01  # type: ignore
    assert acc.loss_fn() == rd.XEnt
