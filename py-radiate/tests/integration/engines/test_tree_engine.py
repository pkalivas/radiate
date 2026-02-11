import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_tree_regression(simple_regression_dataset, random_seed):
    """Test engine with tree codec for regression."""
    inputs, outputs = simple_regression_dataset

    engine = rd.Engine(
        codec=rd.TreeCodec(
            shape=(1, 1),
            vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sub()],
            leaf=[rd.Op.var(0)],
            root=rd.Op.linear(),
        ),
        fitness_func=rd.Regression(inputs, outputs),
        objective=rd.MIN,
        population_size=100,
        offspring_selector=rd.TournamentSelector(3),
        survivor_selector=rd.EliteSelector(),
        alters=[rd.TreeCrossover(0.5), rd.HoistMutator(0.1)],
    )

    result = engine.run([rd.ScoreLimit(0.1), rd.GenerationsLimit(300)])

    assert isinstance(result.value(), rd.Tree)
    assert result.score()[0] < 0.1
    assert result.index() <= 300

    # Validate the tree approximates the function f(x) = 2x
    for i in range(5, 10):
        assert abs(result.value().eval([i])[0] - (2 * i)) < 0.001
