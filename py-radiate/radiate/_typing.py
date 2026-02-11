from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any, Callable, TYPE_CHECKING

from radiate.gp.op import Op

from .engine.handlers import EventHandler
from .dtype import DataType, DataTypeClass

if TYPE_CHECKING:
    from radiate._dependencies import numpy as np  # type: ignore[import]


type RdDataType = DataType | DataTypeClass

type Subscriber = (
    Callable[[Any], None]
    | list[Callable[[Any], None]]
    | EventHandler
    | list[EventHandler]
)

type IntDecoding = int | list[int] | list[list[int]] | "np.ndarray"
type FloatDecoding = float | list[float] | list[list[float]] | "np.ndarray"
type BoolDecoding = bool | list[bool] | list[list[bool]] | "np.ndarray"
type StringDecoding = str | list[str] | list[list[str]]

type NodeValues = list[Op] | Op

type NodeValues = Op | Sequence[Op]
type OpsMap = Mapping[str, Sequence[Op]]  # external view
type OpsDict = dict[str, list[Op]]  # internal canonical form
