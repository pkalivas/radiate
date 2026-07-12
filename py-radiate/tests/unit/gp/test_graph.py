import numpy as np
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

    inputs = np.array([[1.0, 2.0]])

    assert graph is not None
    assert isinstance(graph, rd.Graph)
    assert graph.eval(inputs) is not None


@pytest.mark.unit
def test_gp_graph_eval():
    """Test GP Graph evaluation."""
    # Create a simple graph
    codec = rd.GraphCodec.directed(
        shape=(3, 1),
        vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    graph = codec.decode(codec.encode())

    single_result = graph.eval(np.array([[1.0, 2.0, 3.0]]))

    assert isinstance(single_result, np.ndarray)
    assert single_result.shape[0] == 1
    assert isinstance(single_result[0], np.ndarray)

    multi_result = graph.eval(
        np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]], dtype=float)
    )

    assert isinstance(multi_result, np.ndarray)
    assert multi_result.shape[0] == 3
    assert isinstance(multi_result[0], np.ndarray)
    assert multi_result.shape[1] == 1  # Ensure the output shape is correct


@pytest.mark.integration
def test_graph_from_json(graph_simple_2x1):
    """Test GP Graph creation from JSON."""
    initial_eval = graph_simple_2x1.eval(np.array([[1.0, 2.0]]))
    json_data = graph_simple_2x1.to_json()

    graph = rd.Graph.from_json(json_data)
    post_eval = graph.eval([[1.0, 2.0]])

    assert graph is not None
    assert isinstance(graph, rd.Graph)
    assert initial_eval == post_eval, (
        "Graph evaluation should be consistent before and after JSON serialization"
    )
