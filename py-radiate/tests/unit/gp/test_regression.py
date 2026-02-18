# tests/test_normalize_regression.py
from __future__ import annotations

import math
import pytest

import radiate as rd
from radiate.utils._normalize import _normalize_regression_data, _to_2d_f32


def _is_2d_list(x):
    return isinstance(x, list) and (len(x) == 0 or isinstance(x[0], list))


def _assert_2d_float_lists(x, name: str):
    assert _is_2d_list(x), f"{name} must be list[list[float]], got {type(x)}"
    for row in x:
        assert isinstance(row, list), f"{name} row must be list, got {type(row)}"
        for v in row:
            assert isinstance(v, float), f"{name} values must be float, got {type(v)}"
            assert math.isfinite(v), f"{name} value must be finite, got {v}"


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_polars_df_target_and_feature_cols_smoke(graph_1x1_engine):
    import polars as pl

    inputs = [0.0, 1.0, 2.0, 3.0]
    answers = [0.0, 2.0, 4.0, 6.0]

    df = pl.DataFrame({"dd": inputs, "x": answers, "other": [0.42222] * len(inputs)})

    engine = graph_1x1_engine.regression(
        df, target="x", feature_cols=["dd"], loss="mse"
    )

    res = next(engine)
    assert res.index() == 1


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_polars_df_default_target_last_col_smoke(graph_1x1_engine):
    import polars as pl

    df = pl.DataFrame({"a": [1.0, 2.0, 3.0], "x": [10.0, 20.0, 30.0]})

    engine = graph_1x1_engine.regression(df, loss="mse")

    assert next(engine).index() == 1


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_engine_regression_pandas_df_target_and_feature_cols_smoke(graph_1x1_engine):
    import pandas as pd

    inputs = [0.0, 1.0, 2.0, 3.0]
    answers = [0.0, 2.0, 4.0, 6.0]

    df = pd.DataFrame({"dd": inputs, "x": answers, "other": [0.42222] * len(inputs)})

    engine = graph_1x1_engine.regression(
        df, target="x", feature_cols=["dd"], loss="mse"
    )

    assert next(engine).index() == 1


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_engine_regression_numpy_arrays_smoke(graph_1x1_engine):
    import numpy as np

    X = np.array([[0.0], [1.0], [2.0], [3.0]], dtype=np.float64)
    y = np.array([0.0, 2.0, 4.0, 6.0], dtype=np.float64)

    engine = graph_1x1_engine.regression(X, y, loss="mse")

    assert next(engine).index() == 1


@pytest.mark.unit
def test_engine_regression_python_lists_smoke(graph_1x1_engine):

    X = [[0.0], [1.0], [2.0], [3.0]]
    y = [0.0, 2.0, 4.0, 6.0]

    engine = graph_1x1_engine.regression(X, y, loss="mse")

    assert next(engine).index() == 1


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_invalid_feature_col_raises(graph_1x1_engine):
    import polars as pl

    df = pl.DataFrame({"dd": [1.0, 2.0], "x": [3.0, 4.0]})

    with pytest.raises(Exception):
        graph_1x1_engine.regression(
            df, target="x", feature_cols=["does_not_exist"], loss="mse"
        )


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_invalid_target_col_raises(graph_1x1_engine):
    import polars as pl

    df = pl.DataFrame({"dd": [1.0, 2.0], "x": [3.0, 4.0]})

    with pytest.raises(Exception):
        graph_1x1_engine.regression(df, target="nope", feature_cols=["dd"], loss="mse")


@pytest.mark.unit
def test_to_2d_f32_rejects_str_bytes_none():
    with pytest.raises(TypeError):
        _to_2d_f32("abc", name="features")
    with pytest.raises(TypeError):
        _to_2d_f32(b"abc", name="features")
    with pytest.raises(TypeError):
        _to_2d_f32(None, name="features")


@pytest.mark.unit
def test_to_2d_f32_python_1d_list_becomes_column_vector():
    x = [1, 2, 3]
    out = _to_2d_f32(x, name="features")
    assert out == [[1.0], [2.0], [3.0]]


@pytest.mark.unit
def test_to_2d_f32_python_2d_list_is_cast_to_float():
    x = [[1, 2], [3, 4]]
    out = _to_2d_f32(x, name="features")
    assert out == [[1.0, 2.0], [3.0, 4.0]]


@pytest.mark.unit
def test_normalize_regression_with_python_lists_1d_targets():
    X, y = _normalize_regression_data(
        features=[[1, 2], [3, 4]],
        targets=[10, 20],
    )
    # Expect canonical: list[list[float]]
    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")
    assert y == [[10.0], [20.0]]


@pytest.mark.unit
def test_normalize_regression_with_python_lists_2d_targets():
    X, y = _normalize_regression_data(
        features=[[1, 2], [3, 4]],
        targets=[[10], [20]],
    )
    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")
    assert y == [[10.0], [20.0]]


@pytest.mark.parametrize(
    "features,targets",
    [
        ([[1, 2], [3, 4]], None),  # targets missing but features not a dataframe
    ],
)
def test_normalize_regression_errors_when_targets_none_and_not_dataframe(
    features, targets
):
    with pytest.raises(TypeError):
        _normalize_regression_data(features, targets)


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_normalize_regression_numpy_arrays_return_lists_2d_float():
    import numpy as np

    X_np = np.array([[1, 2], [3, 4]], dtype=np.int64)
    y_np = np.array([10, 20], dtype=np.int64)  # 1D

    X, y = _normalize_regression_data(X_np, y_np)

    # Since PyFitnessFn doesn't accept numpy, we expect lists
    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")
    assert X == [[1.0, 2.0], [3.0, 4.0]]
    assert y == [[10.0], [20.0]]


@pytest.mark.unit
def test_normalize_regression_rejects_string_inputs():
    with pytest.raises(TypeError):
        _normalize_regression_data("not array-like", [1, 2])
    with pytest.raises(TypeError):
        _normalize_regression_data([1, 2], "not array-like")


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_normalize_regression_pandas_dataframe_default_target_last_column():
    import pandas as pd

    df = pd.DataFrame(
        {
            "x1": [1, 2, 3],
            "x2": [4, 5, 6],
            "y": [10, 20, 30],
        }
    )

    X, y = _normalize_regression_data(df)

    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")

    assert X == [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]
    assert y == [[10.0], [20.0], [30.0]]


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_normalize_regression_pandas_dataframe_target_col_and_feature_cols():
    import pandas as pd

    df = pd.DataFrame(
        {
            "a": [1, 2],
            "b": [3, 4],
            "y": [10, 20],
        }
    )

    X, y = _normalize_regression_data(df, target_col="y", feature_cols=["b"])

    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")

    assert X == [[3.0], [4.0]]
    assert y == [[10.0], [20.0]]


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_normalize_regression_polars_dataframe_default_target_last_column():
    import polars as pl

    df = pl.DataFrame(
        {
            "x1": [1, 2, 3],
            "x2": [4, 5, 6],
            "y": [10, 20, 30],
        }
    )

    X, y = _normalize_regression_data(df)

    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")

    assert X == [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]
    assert y == [[10.0], [20.0], [30.0]]


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_normalize_regression_polars_dataframe_target_col_and_feature_cols():
    import polars as pl

    df = pl.DataFrame(
        {
            "a": [1, 2],
            "b": [3, 4],
            "y": [10, 20],
        }
    )

    X, y = _normalize_regression_data(df, target_col="y", feature_cols=["b"])

    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")

    assert X == [[3.0], [4.0]]
    assert y == [[10.0], [20.0]]


@pytest.mark.skipif(not rd._TORCH_AVAILABLE, reason="PyTorch not installed")
def test_normalize_regression_torch_tensor_inputs():
    import torch

    X_t = torch.tensor([[1, 2], [3, 4]], dtype=torch.int64)
    y_t = torch.tensor([10, 20], dtype=torch.int64)

    X, y = _normalize_regression_data(X_t, y_t)

    _assert_2d_float_lists(X, "features")
    _assert_2d_float_lists(y, "targets")
    assert X == [[1.0, 2.0], [3.0, 4.0]]
    assert y == [[10.0], [20.0]]
