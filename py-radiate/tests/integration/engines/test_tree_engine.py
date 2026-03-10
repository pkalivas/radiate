import radiate as rd
import pytest


@pytest.mark.integration
def test_engine_tree_regression(simple_regression_dataset, random_seed):
    """Test engine with tree codec for regression."""
    inputs, outputs = simple_regression_dataset

    engine = (
        rd.Engine.tree(
            shape=(1, 1),
            vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.sub()],
            leaf=[rd.Op.var(0)],
            root=rd.Op.linear(),
        )
        .regression(inputs, outputs)
        .select(rd.Select.tournament(3), rd.Select.elite())
        .alters(rd.Cross.tree(0.5), rd.Mutate.hoist(0.1))
        .limit(rd.Limit.score(0.01), rd.Limit.generations(500))
    )

    result = engine.run()

    assert isinstance(result.value(), rd.Tree)
    assert result.score()[0] < 0.1
    assert result.index() <= 300

    # Validate the tree approximates the function f(x) = 2x
    for i in range(5, 10):
        assert abs(result.value().eval([i])[0] - (2 * i)) < 0.001
