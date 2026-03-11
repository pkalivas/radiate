from __future__ import annotations

from typing import Any, Sequence

from radiate._dependancies import (
    _check_for_numpy,
    _check_for_pandas,
    _check_for_polars,
    _check_for_torch,
)

from radiate._dependancies import numpy as np
from radiate._dependancies import pandas as pd
from radiate._dependancies import polars as pl
from radiate._dependancies import torch


def _ensure_2d_np(arr: Any, *, name: str) -> Any:
    if arr.ndim == 1:
        return arr.reshape(-1, 1)
    if arr.ndim != 2:
        raise ValueError(f"{name} must be 1D or 2D. Got shape={arr.shape}")
    return arr


def _as_numpy_2d_f32(x: Any, *, name: str) -> Any:
    """
    Convert supported array-like inputs into a 2D numpy float32 ndarray.
    """
    if _check_for_numpy(x) and isinstance(x, np.ndarray):
        return _ensure_2d_np(x.astype(np.float32, copy=False), name=name)

    if _check_for_polars(x):
        if isinstance(x, pl.DataFrame):
            return _ensure_2d_np(x.to_numpy().astype(np.float32, copy=False), name=name)
        if isinstance(x, pl.Series):
            return _ensure_2d_np(x.to_numpy().astype(np.float32, copy=False), name=name)

    if _check_for_torch(x) and isinstance(x, torch.Tensor):
        arr = x.detach().cpu().numpy().astype(np.float32, copy=False)
        return _ensure_2d_np(arr, name=name)

    if _check_for_pandas(x):
        if isinstance(x, pd.DataFrame):
            return _ensure_2d_np(x.to_numpy().astype(np.float32, copy=False), name=name)
        if isinstance(x, pd.Series):
            return _ensure_2d_np(x.to_numpy().astype(np.float32, copy=False), name=name)

    if isinstance(x, (str, bytes)) or x is None:
        raise TypeError(f"{name} must be array-like. Got {type(x).__name__}")

    try:
        first = x[0]
    except Exception as e:
        raise TypeError(f"{name} must be array-like. Got {type(x).__name__}") from e

    # Python sequence fallback
    if not isinstance(first, (str, bytes)) and hasattr(first, "__iter__"):
        arr = np.array([[float(v) for v in row] for row in x], dtype=np.float32)
    else:
        arr = np.array([[float(v)] for v in x], dtype=np.float32)

    return _ensure_2d_np(arr, name=name)


def _to_2d_f32(x: Any, *, name: str) -> list[list[float]]:
    """
    Normalize x into a 2D nested float list.

    Supports:
      - python sequences
      - numpy arrays
      - polars DataFrame / Series
      - pandas DataFrame / Series
      - torch tensors
    """
    return _as_numpy_2d_f32(x, name=name).tolist()


def _normalize_single_chunk(
    features: Any,
    *,
    cols: Sequence[Any] | None = None,
) -> list[list[float]]:
    """
    Normalize a single evaluation chunk to list[list[float]].

    If cols is provided:
      - polars/pandas DataFrames: select columns by name/index
      - numpy arrays: select columns by numeric indices/slices only
    """
    if cols is not None:
        if _check_for_polars(features) and isinstance(features, pl.DataFrame):
            features = features.select(list(cols))

        elif _check_for_pandas(features) and isinstance(features, pd.DataFrame):
            features = features[list(cols)]

        elif _check_for_numpy(features) and isinstance(features, np.ndarray):
            features = features[:, cols]

        else:
            raise TypeError(
                "Unsupported features type for column selection; expected "
                "polars.DataFrame, pandas.DataFrame, or numpy.ndarray."
            )

    return _to_2d_f32(features, name="features")


def _normalize_regression_data(
    features: Any,
    targets: Any | None = None,
    *,
    feature_cols: Sequence[Any] | None = None,
    target_cols: Sequence[Any] | None = None,
) -> tuple[list[list[float]], list[list[float]]]:
    """
    Normalize regression inputs into (X, y), both as list[list[float]].

    Cases:
      1. features is a DataFrame and targets is None:
         - infer X/y from target_cols or use last column as target
      2. features and targets are passed separately:
         - normalize each independently
    """
    if targets is None:
        if _check_for_polars(features) and isinstance(features, pl.DataFrame):
            df = features

            if target_cols is None:
                if len(df.columns) < 2:
                    raise ValueError(
                        "Regression dataframe must contain at least one feature column "
                        "and one target column."
                    )
                X = df.select(df.columns[:-1])
                y = df.select([df.columns[-1]])
            else:
                target_cols = list(target_cols)
                cols = (
                    list(feature_cols)
                    if feature_cols is not None
                    else [c for c in df.columns if c not in target_cols]
                )

                if not cols:
                    raise ValueError(
                        "No feature columns remain after excluding target_cols."
                    )
                if not target_cols:
                    raise ValueError("target_cols cannot be empty.")

                X = df.select(cols)
                y = df.select(target_cols)

        elif _check_for_pandas(features) and isinstance(features, pd.DataFrame):
            df = features

            if target_cols is None:
                if df.shape[1] < 2:
                    raise ValueError(
                        "Regression dataframe must contain at least one feature column "
                        "and one target column."
                    )
                X = df.iloc[:, :-1]
                y = df.iloc[:, -1:]
            else:
                target_cols = list(target_cols)
                cols = (
                    list(feature_cols)
                    if feature_cols is not None
                    else [c for c in df.columns if c not in target_cols]
                )

                if not cols:
                    raise ValueError(
                        "No feature columns remain after excluding target_cols."
                    )
                if not target_cols:
                    raise ValueError("target_cols cannot be empty.")

                X = df[cols]
                y = df[target_cols]

        else:
            raise TypeError(
                "When targets is None, features must be a polars or pandas DataFrame."
            )
    else:
        X = features
        y = targets

    X_out = _to_2d_f32(X, name="features")
    y_out = _to_2d_f32(y, name="targets")

    if len(X_out) != len(y_out):
        raise ValueError(
            f"features and targets must have the same number of rows. "
            f"Got {len(X_out)} and {len(y_out)}."
        )

    return X_out, y_out
