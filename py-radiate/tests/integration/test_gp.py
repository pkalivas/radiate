from __future__ import annotations

import pytest

import radiate as rd


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_normalize_graph_eval_with_pl_dataframe(graph_simple_2x1, random_seed):
    import polars as pl

    df = pl.DataFrame(
        {
            "x1": [1, 2, 3],
            "x2": [4, 5, 6],
            "y": [10, 20, 30],
        }
    )

    predictions = graph_simple_2x1.eval(df, columns=["x1", "x2"])

    assert len(predictions) == 3
    assert all(isinstance(pred, list) and len(pred) == 1 for pred in predictions)
    assert all(isinstance(pred[0], float) for pred in predictions)


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_normalize_graph_eval_with_pl_series(graph_simple_1x1, random_seed):
    import polars as pl

    series = pl.Series("x1", [1, 2, 3])

    predictions = graph_simple_1x1.eval(series)

    assert len(predictions) == 3
    assert all(isinstance(pred, list) and len(pred) == 1 for pred in predictions)
    assert all(isinstance(pred[0], float) for pred in predictions)


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_normalize_graph_eval_with_pandas_dataframe(graph_simple_2x1, random_seed):
    import pandas as pd

    df = pd.DataFrame(
        {
            "x1": [1, 2, 3],
            "x2": [4, 5, 6],
            "y": [10, 20, 30],
        }
    )

    predictions = graph_simple_2x1.eval(df, columns=["x1", "x2"])

    assert len(predictions) == 3
    assert all(isinstance(pred, list) and len(pred) == 1 for pred in predictions)
    assert all(isinstance(pred[0], float) for pred in predictions)


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_normalize_graph_eval_with_pandas_series(graph_simple_1x1, random_seed):
    import pandas as pd

    series = pd.Series([1, 2, 3], name="x1")

    predictions = graph_simple_1x1.eval(series)

    assert len(predictions) == 3
    assert all(isinstance(pred, list) and len(pred) == 1 for pred in predictions)
    assert all(isinstance(pred[0], float) for pred in predictions)


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_normalize_graph_eval_with_numpy_array(graph_simple_2x1, random_seed):
    import numpy as np

    arr = np.array([[1, 4], [2, 5], [3, 6]])

    predictions = graph_simple_2x1.eval(arr)

    assert len(predictions) == 3
    assert all(isinstance(pred, list) and len(pred) == 1 for pred in predictions)
    assert all(isinstance(pred[0], float) for pred in predictions)


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_normalize_graph_eval_with_numpy_1d_array(graph_simple_1x1, random_seed):
    import numpy as np

    arr = np.array([1, 2, 3])

    predictions = graph_simple_1x1.eval(arr)

    assert len(predictions) == 3
    assert all(isinstance(pred, list) and len(pred) == 1 for pred in predictions)
    assert all(isinstance(pred[0], float) for pred in predictions)
