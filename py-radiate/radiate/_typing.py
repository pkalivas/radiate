from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any, Callable, Literal, TYPE_CHECKING

from radiate.gp.op import Op

from .handlers import EventHandler
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

type IntDecoding = int | list[int] | list[list[int]]
type FloatDecoding = float | list[float] | list[list[float]]
type BoolDecoding = bool | list[bool] | list[list[bool]]
type StringDecoding = str | list[str] | list[list[str]]


type NodeValues = list[Op] | Op
type GraphNodeTypes = Literal["input", "vertex", "edge", "output"]
type TreeNodeTypes = Literal["root", "function", "terminal"]


type NodeValues = Op | Sequence[Op]
type OpsMap = Mapping[str, Sequence[Op]]  # external view
type OpsDict = dict[str, list[Op]]  # internal canonical form
