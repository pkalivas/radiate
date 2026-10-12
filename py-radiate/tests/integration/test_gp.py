from __future__ import annotations

import pytest

import radiate as rd


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_normalize_graph_eval_with_pl_dataframe(random_seed):
    import polars as pl

    df = pl.DataFrame(
        {
            "x1": [1.0, 2.0, 3.0],
            "x2": [4.0, 5.0, 6.0],
            "y": [10.0, 20.0, 30.0],
        }
    )

    codec = rd.GraphCodec.directed(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    graph = codec.decode(codec.encode())
    predictions = graph.eval(df, columns=["x1", "x2"])

    assert len(predictions) == 3


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_normalize_graph_eval_with_pandas_dataframe(random_seed):
    import pandas as pd

    df = pd.DataFrame(
        {
            "x1": [1.0, 2.0, 3.0],
            "x2": [4.0, 5.0, 6.0],
            "y": [10.0, 20.0, 30.0],
        }
    )

    codec = rd.GraphCodec.directed(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    graph = codec.decode(codec.encode())

    predictions = graph.eval(df, columns=["x1", "x2"])

    assert len(predictions) == 3
    assert predictions.shape == (
        3,
        1,
    )


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_normalize_graph_eval_with_numpy_array(random_seed):
    import numpy as np

    arr = np.array([[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]], dtype=np.float64)
    codec = rd.GraphCodec.directed(
        shape=(2, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    graph = codec.decode(codec.encode())

    predictions = graph.eval(arr)

    assert len(predictions) == 3
    assert predictions.shape == (
        3,
        1,
    )


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_normalize_graph_eval_with_numpy_1d_array(random_seed):
    import numpy as np

    arr = np.array([1, 2, 3], dtype=np.float64).reshape(-1, 1)
    codec = rd.GraphCodec.directed(
        shape=(1, 1),
        vertex=[rd.Op.add(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )

    graph = codec.decode(codec.encode())
    predictions = graph.eval(arr)

    assert len(predictions) == 3
    assert predictions.shape == (
        3,
        1,
    )
