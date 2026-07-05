from __future__ import annotations

import pytest

import radiate as rd


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_normalize_graph_eval_with_pl_dataframe(graph_simple_2x1, random_seed):
    import polars as pl

    df = pl.DataFrame(
        {
            "x1": [1.0, 2.0, 3.0],
            "x2": [4.0, 5.0, 6.0],
            "y": [10.0, 20.0, 30.0],
        }
    )

    predictions = graph_simple_2x1.eval(df, columns=["x1", "x2"])

    assert len(predictions) == 3


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_normalize_graph_eval_with_pandas_dataframe(graph_simple_2x1, random_seed):
    import pandas as pd

    df = pd.DataFrame(
        {
            "x1": [1.0, 2.0, 3.0],
            "x2": [4.0, 5.0, 6.0],
            "y": [10.0, 20.0, 30.0],
        }
    )

    predictions = graph_simple_2x1.eval(df, columns=["x1", "x2"])

    assert len(predictions) == 3
    assert predictions.shape == (
        3,
        1,
    )


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_normalize_graph_eval_with_numpy_array(graph_simple_2x1, random_seed):
    import numpy as np

    arr = np.array([[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]], dtype=np.float32)

    predictions = graph_simple_2x1.eval(arr)

    assert len(predictions) == 3
    assert predictions.shape == (
        3,
        1,
    )


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_normalize_graph_eval_with_numpy_1d_array(graph_simple_1x1, random_seed):
    import numpy as np

    arr = np.array([1, 2, 3], dtype=np.float32).reshape(-1, 1)

    predictions = graph_simple_1x1.eval(arr)

    assert len(predictions) == 3
    assert predictions.shape == (
        3,
        1,
    )
