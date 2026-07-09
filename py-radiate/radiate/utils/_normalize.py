from __future__ import annotations

from typing import Any, Sequence

from .._dependancies import (
    _NUMPY_AVAILABLE,
    _PANDAS_AVAILABLE,
    _POLARS_AVAILABLE,
    _check_for_numpy,
    _check_for_pandas,
    _check_for_polars,
    _check_for_torch,
    torch,
)
from .._dependancies import numpy as np
from .._dependancies import pandas as pd
from .._dependancies import polars as pl


def _select_columns(df: Any, columns: Sequence[Any]) -> Any:
    """Select columns (by name) from a polars or pandas DataFrame."""
    if _check_for_polars(df) and isinstance(df, pl.DataFrame):
        return df.select(list(columns))
    if _check_for_pandas(df) and isinstance(df, pd.DataFrame):
        return df[list(columns)]
    raise TypeError(
        f"columns is only supported for DataFrame input, got {type(df).__name__}"
    )


def _to_float_array(x: Any, *, columns: Sequence[Any] | None = None) -> "np.ndarray":
    """
    Normalize a DataFrame/Series/tensor/array-like into a contiguous NumPy
    array. Whatever float width the input already has (float32/float64) is
    preserved; anything else (int arrays, plain Python sequences — which have
    no native width of their own) becomes float64. `columns` selects a
    DataFrame's columns before conversion and only makes sense for DataFrame
    input; passing it alongside anything else is an error, not a silent no-op.
    """
    # Fast path: already exactly what a tight eval loop hands in on every call
    # after the first — skip the DataFrame/Series/tensor dispatch entirely.
    if (
        columns is None
        and _check_for_numpy(x)
        and getattr(x, "dtype", None) in (np.float32, np.float64)
    ):
        return x if x.flags["C_CONTIGUOUS"] else np.ascontiguousarray(x)

    is_polars = _check_for_polars(x)
    is_pandas = _check_for_pandas(x)

    is_series = (is_polars and isinstance(x, pl.Series)) or (
        is_pandas and isinstance(x, pd.Series)
    )

    if is_series and _NUMPY_AVAILABLE:
        if columns is not None:
            raise TypeError("columns is not supported for a Series input")
        arr = x.to_numpy()
    elif (is_polars and isinstance(x, pl.DataFrame)) or (
        is_pandas and isinstance(x, pd.DataFrame)
    ):
        arr = (_select_columns(x, columns) if columns is not None else x).to_numpy()
    elif _check_for_torch(x) and isinstance(x, torch.Tensor):
        if columns is not None:
            raise TypeError("columns is only supported for DataFrame input")
        arr = x.detach().cpu().numpy()
    else:
        if columns is not None:
            raise TypeError(
                f"columns is only supported for DataFrame input, got {type(x).__name__}"
            )
        if isinstance(x, (str, bytes)) or x is None:
            raise TypeError(f"must be array-like. Got {type(x).__name__}")
        arr = np.asarray(x)

    if arr.dtype not in (np.float32, np.float64):
        arr = arr.astype(np.float64)
    if not arr.flags["C_CONTIGUOUS"]:
        arr = np.ascontiguousarray(arr)

    return arr


def _reshape_for_regression(arr: "np.ndarray", *, name: str) -> "np.ndarray":
    """A 1D regression column is N samples of one feature, not one N-feature sample."""
    if _check_for_numpy(arr):
        if arr.ndim == 1:
            return arr.reshape(-1, 1)
        if arr.ndim != 2:
            raise ValueError(f"{name} must be 1D or 2D. Got shape={arr.shape}")
    return arr


def _split_feature_target_columns(
    columns: Sequence[Any],
    *,
    feature_cols: Sequence[Any] | None,
    target_cols: Sequence[Any] | None,
) -> tuple[list[Any], list[Any]]:
    """Resolve (feature_cols, target_cols) name lists from one DataFrame's columns."""
    if target_cols is None:
        if len(columns) < 2:
            raise ValueError(
                "Regression dataframe must contain at least one feature column "
                "and one target column."
            )
        return list(columns[:-1]), [columns[-1]]

    target_cols = list(target_cols)
    feature_cols = (
        list(feature_cols)
        if feature_cols is not None
        else [c for c in columns if c not in target_cols]
    )

    if not feature_cols:
        raise ValueError("No feature columns remain after excluding target_cols.")
    if not target_cols:
        raise ValueError("target_cols cannot be empty.")

    return feature_cols, target_cols


def _normalize_regression_data(
    features: Any,
    targets: Any | None = None,
    *,
    feature_cols: Sequence[Any] | None = None,
    target_cols: Sequence[Any] | None = None,
) -> tuple[np.ndarray, np.ndarray]:
    """
    Normalize regression inputs into (X, y), each a contiguous NumPy array
    that preserves its original float width (see `_to_float_array`).

    Cases:
      1. features is a DataFrame and targets is None:
         - infer X/y from target_cols or use the last column as the target
      2. features and targets are passed separately:
         - normalize each independently
    """
    if targets is None:
        if _check_for_polars(features) and isinstance(features, pl.DataFrame):
            columns = features.columns
        elif _check_for_pandas(features) and isinstance(features, pd.DataFrame):
            columns = list(features.columns)
        else:
            raise TypeError(
                "When targets is None, features must be a polars or pandas DataFrame."
            )

        resolved_feature_cols, resolved_target_cols = _split_feature_target_columns(
            columns, feature_cols=feature_cols, target_cols=target_cols
        )
        X = _to_float_array(features, columns=resolved_feature_cols)
        y = _to_float_array(features, columns=resolved_target_cols)
    else:
        X = _to_float_array(features, columns=feature_cols)
        y = _to_float_array(targets, columns=target_cols)

    X = _reshape_for_regression(X, name="features")
    y = _reshape_for_regression(y, name="targets")

    if len(X) != len(y):
        raise ValueError(
            f"features and targets must have the same number of rows. "
            f"Got {len(X)} and {len(y)}."
        )

    return X, y
