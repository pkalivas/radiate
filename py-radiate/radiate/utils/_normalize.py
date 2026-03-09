from __future__ import annotations

from typing import Any
from radiate._dependancies import (
    _NUMPY_AVAILABLE,
    _POLARS_AVAILABLE,
    _PANDAS_AVAILABLE,
    _TORCH_AVAILABLE,
    _check_for_numpy,
    _check_for_polars,
    _check_for_pandas,
    _check_for_torch,
)

from radiate._dependancies import numpy as np
from radiate._dependancies import pandas as pd
from radiate._dependancies import polars as pl
from radiate._dependancies import torch


def _ensure_2d_np(arr: Any, *, name: str) -> Any:
    # arr is a numpy ndarray *already*
    if arr.ndim == 1:
        return arr.reshape(-1, 1)
    if arr.ndim != 2:
        raise ValueError(f"{name} must be 1D or 2D. Got shape={arr.shape}")
    return arr


def _to_2d_f32(x: Any, *, name: str) -> Any:
    """
    Normalize x into either:
      - numpy ndarray float32 (preferred when numpy installed), or
      - list[list[float]] fallback when numpy isn't installed.

    Supports: python seqs, numpy arrays, polars DF/Series, pandas DF/Series, torch tensors.
    """

    if _check_for_numpy(x):
        import numpy as np

        if isinstance(x, np.ndarray):
            arr = x.astype(np.float32, copy=False)
            return _ensure_2d_np(arr, name=name)

    if _check_for_polars(x):
        import polars as pl

        if isinstance(x, pl.DataFrame):
            x = x.to_numpy()
        elif isinstance(x, pl.Series):
            x = x.to_numpy()

    if _check_for_torch(x):
        import torch

        if isinstance(x, torch.Tensor):
            x = x.detach().cpu().numpy()

    if _check_for_pandas(x):
        import pandas as pd

        if isinstance(x, pd.DataFrame) or isinstance(x, pd.Series):
            x = x.to_numpy()

    # Accept 1D: [1,2,3] -> [[1],[2],[3]]
    # Accept 2D: [[...],[...]]
    if isinstance(x, (str, bytes)) or x is None:
        raise TypeError(f"{name} must be array-like. Got {type(x).__name__}")

    try:
        first = x[0]
    except Exception as e:
        raise TypeError(f"{name} must be array-like. Got {type(x).__name__}") from e

    # 2D if first element is itself indexable (but not str/bytes)
    if not isinstance(first, (str, bytes)) and hasattr(first, "__iter__"):
        return [[float(v) for v in row] for row in x]  # type: ignore[arg-type]
    else:
        return [[float(v)] for v in x]  # type: ignore[arg-type]


def _normalize_regression_data(
    features: Any,
    targets: Any | None = None,
    *,
    feature_cols=None,
    target_cols=None,
):
    if targets is None:
        if _check_for_polars(features):
            df = features
            if target_cols is None:
                X = df.select(df.columns[:-1])
                y = df.select([df.columns[-1]])  # force 1-col DF
            else:
                cols = feature_cols or [c for c in df.columns if c not in target_cols]
                X = df.select(cols)
                y = df.select(target_cols)  # force 1-col DF

            # convert after selection
            X = X.to_numpy()
            y = y.to_numpy()

        elif _check_for_pandas(features):
            df = features
            if target_cols is None:
                X = df.iloc[:, :-1].to_numpy()
                y = df.iloc[:, -1:].to_numpy()  # already 2D
            else:
                cols = feature_cols or [c for c in df.columns if c not in target_cols]
                X = df[cols].to_numpy()
                y = df[target_cols].to_numpy()  # force 2D
        else:
            raise TypeError("Unsupported dataframe type for regression")
    else:
        X = features
        y = targets

    X = _to_2d_f32(X, name="features")
    y = _to_2d_f32(y, name="targets")

    if _check_for_numpy(X):
        X = X.tolist()
    if _check_for_numpy(y):
        y = y.tolist()

    return X, y
