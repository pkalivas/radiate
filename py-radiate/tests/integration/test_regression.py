from __future__ import annotations

import numpy as np
import pytest

import radiate as rd
from radiate.utils._normalize import _normalize_regression_data, _to_float_array


def _assert_2d_float_array(x, name: str):
    assert isinstance(x, np.ndarray), f"{name} must be a NumPy array, got {type(x)}"
    assert x.ndim == 2, f"{name} must be 2D, got shape={x.shape}"
    assert x.dtype in (np.float32, np.float64), f"{name} must be float, got {x.dtype}"
    assert np.isfinite(x).all(), f"{name} must be all finite, got {x}"


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_polars_df_target_and_feature_cols_smoke(graph_1x1_engine):
    import polars as pl

    inputs = [0.0, 1.0, 2.0, 3.0]
    answers = [0.0, 2.0, 4.0, 6.0]

    df = pl.DataFrame({"dd": inputs, "x": answers, "other": [0.42222] * len(inputs)})

    engine = graph_1x1_engine.regression(
        df, target_cols="x", feature_cols=["dd"], loss=rd.MSE
    ).limit(rd.Limit.generations(10))

    res = next(engine)
    assert res.index() == 1


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_polars_df_default_target_last_col_smoke(graph_1x1_engine):
    import polars as pl

    df = pl.DataFrame({"a": [1.0, 2.0, 3.0], "x": [10.0, 20.0, 30.0]})

    engine = graph_1x1_engine.regression(df, loss=rd.MSE).limit(
        rd.Limit.generations(10)
    )

    assert next(engine).index() == 1


@pytest.mark.skipif(not rd._PANDAS_AVAILABLE, reason="pandas not installed")
def test_engine_regression_pandas_df_target_and_feature_cols_smoke(graph_1x1_engine):
    import pandas as pd

    inputs = [0.0, 1.0, 2.0, 3.0]
    answers = [0.0, 2.0, 4.0, 6.0]

    df = pd.DataFrame({"dd": inputs, "x": answers, "other": [0.42222] * len(inputs)})

    engine = graph_1x1_engine.regression(
        df, target_cols="x", feature_cols=["dd"], loss=rd.MSE
    ).limit(rd.Limit.generations(10))

    assert next(engine).index() == 1


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_engine_regression_numpy_arrays_smoke(graph_1x1_engine):
    import numpy as np

    X = np.array([[0.0], [1.0], [2.0], [3.0]], dtype=np.float64)
    y = np.array([0.0, 2.0, 4.0, 6.0], dtype=np.float64)

    engine = graph_1x1_engine.regression(X, y, loss=rd.MSE).limit(
        rd.Limit.generations(10)
    )

    assert X.shape == (4, 1)
    assert y.shape == (4,)

    assert next(engine).index() == 1


@pytest.mark.skipif(not rd._NUMPY_AVAILABLE, reason="numpy not installed")
def test_engine_regression_python_lists_smoke(graph_1x1_engine):
    X = [[0.0], [1.0], [2.0], [3.0]]
    y = [0.0, 2.0, 4.0, 6.0]

    engine = graph_1x1_engine.regression(X, y, loss=rd.MSE).limit(
        rd.Limit.generations(10)
    )

    assert next(engine).index() == 1


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_invalid_feature_col_raises(graph_1x1_engine):
    import polars as pl

    df = pl.DataFrame({"dd": [1.0, 2.0], "x": [3.0, 4.0]})

    with pytest.raises(Exception):
        graph_1x1_engine.regression(
            df, target="x", feature_cols=["does_not_exist"], loss=rd.MSE
        )


@pytest.mark.skipif(not rd._POLARS_AVAILABLE, reason="polars not installed")
def test_engine_regression_invalid_target_col_raises(graph_1x1_engine):
    import polars as pl

    df = pl.DataFrame({"dd": [1.0, 2.0], "x": [3.0, 4.0]})

    with pytest.raises(Exception):
        graph_1x1_engine.regression(df, target="nope", feature_cols=["dd"], loss=rd.MSE)


@pytest.mark.unit
def test_to_float_array_rejects_str_bytes_none():
    with pytest.raises(TypeError):
        _to_float_array("abc")
    with pytest.raises(TypeError):
        _to_float_array(b"abc")
    with pytest.raises(TypeError):
        _to_float_array(None)


@pytest.mark.unit
def test_to_float_array_python_1d_list_stays_1d():
    x = [1, 2, 3]
    out = _to_float_array(x)
    assert out.tolist() == [1.0, 2.0, 3.0]
    assert out.dtype == np.float64


@pytest.mark.unit
def test_to_float_array_python_2d_list_is_cast_to_float():
    x = [[1, 2], [3, 4]]
    out = _to_float_array(x)
    assert out.tolist() == [[1.0, 2.0], [3.0, 4.0]]
    assert out.dtype == np.float64


@pytest.mark.unit
def test_to_float_array_preserves_existing_float_width():
    x = np.array([[1.0, 2.0]], dtype=np.float32)
    assert _to_float_array(x).dtype == np.float32
    assert _to_float_array(x.astype(np.float64)).dtype == np.float64


@pytest.mark.unit
def test_normalize_regression_with_python_lists_1d_targets():
    X, y = _normalize_regression_data(
        features=[[1, 2], [3, 4]],
        targets=[[10], [20]],
    )
    # Expect canonical: 2D float NumPy arrays
    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")
    assert y.tolist() == [[10.0], [20.0]]


@pytest.mark.unit
def test_normalize_regression_with_python_lists_2d_targets():
    X, y = _normalize_regression_data(
        features=[[1, 2], [3, 4]],
        targets=[[10], [20]],
    )
    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")
    assert y.tolist() == [[10.0], [20.0]]


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


@pytest.mark.integration
def test_normalize_regression_numpy_arrays_preserve_dtype():
    X_np = np.array([[1, 2], [3, 4]], dtype=np.int64)
    y_np = np.array([10, 20], dtype=np.int64)  # 1D

    X, y = _normalize_regression_data(X_np, y_np)

    # int arrays have no native float width, so they widen to float64
    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")
    assert X.tolist() == [[1.0, 2.0], [3.0, 4.0]]
    assert y.tolist() == [[10.0], [20.0]]


def test_normalize_regression_rejects_string_inputs():
    with pytest.raises(TypeError):
        _normalize_regression_data("not array-like", [1, 2])
    with pytest.raises(TypeError):
        _normalize_regression_data([1, 2], "not array-like")


@pytest.mark.skipif(
    not rd._PANDAS_AVAILABLE and not rd._NUMPY_AVAILABLE, reason="pandas not installed"
)
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

    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")

    assert X.tolist() == [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]
    assert y.tolist() == [[10.0], [20.0], [30.0]]


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

    X, y = _normalize_regression_data(df, target_cols="y", feature_cols=["b"])

    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")

    assert X.tolist() == [[3.0], [4.0]]
    assert y.tolist() == [[10.0], [20.0]]


@pytest.mark.skipif(
    not rd._POLARS_AVAILABLE and not rd._NUMPY_AVAILABLE, reason="polars not installed"
)
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

    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")

    assert X.tolist() == [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]
    assert y.tolist() == [[10.0], [20.0], [30.0]]


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

    X, y = _normalize_regression_data(df, target_cols="y", feature_cols=["b"])

    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")

    assert X.tolist() == [[3.0], [4.0]]
    assert y.tolist() == [[10.0], [20.0]]


@pytest.mark.skipif(not rd._TORCH_AVAILABLE, reason="PyTorch not installed")
def test_normalize_regression_torch_tensor_inputs():
    import torch

    X_t = torch.tensor([[1, 2], [3, 4]], dtype=torch.int64)
    y_t = torch.tensor([[10], [20]], dtype=torch.int64)

    X, y = _normalize_regression_data(X_t, y_t)

    _assert_2d_float_array(X, "features")
    _assert_2d_float_array(y, "targets")
    assert X.tolist() == [[1.0, 2.0], [3.0, 4.0]]
    assert y.tolist() == [[10.0], [20.0]]
