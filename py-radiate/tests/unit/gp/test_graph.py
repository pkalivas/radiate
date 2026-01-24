import pytest
import radiate as rd


@pytest.mark.unit
def test_gp_graph_creation():
    """Test GP Graph creation."""
    codec = rd.GraphCodec.directed(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
        edge=[rd.Op.weight()],
        output=[rd.Op.linear()],
    )

    graph = codec.decode(codec.encode())

    assert graph is not None
    assert isinstance(graph, rd.Graph)
    assert graph.eval([[1.0, 2.0]]) is not None


@pytest.mark.unit
def test_gp_graph_eval():
    """Test GP Graph evaluation."""
    # Create a simple graph
    codec = rd.GraphCodec.directed(
        shape=(3, 1),
        vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
        edge=[rd.Op.weight()],
        output=[rd.Op.linear()],
    )

    graph = codec.decode(codec.encode())

    result = graph.eval([[1.0, 2.0, 3.0]])

    assert isinstance(result, list)
    assert len(result) == 1

    multi_result = graph.eval(
        [
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ]
    )

    assert isinstance(multi_result, list)
    assert len(multi_result) == 3


@pytest.mark.integration
def test_graph_from_json(graph_simple_2x1):
    """Test GP Graph creation from JSON."""
    json_data = graph_simple_2x1.to_json()
    graph = rd.Graph.from_json(json_data)

    assert graph is not None
    assert isinstance(graph, rd.Graph)
    assert graph.eval([[1.0, 2.0]]) is not None


@pytest.mark.unit
def test_gp_graph_eval_with_invalid_input(graph_simple_2x1):
    """Test GP Graph evaluation with invalid input."""
    with pytest.raises(TypeError):
        graph_simple_2x1.eval("invalid_input")
    with pytest.raises(TypeError):
        graph_simple_2x1.eval([[1.0, "invalid", 3.0]])
