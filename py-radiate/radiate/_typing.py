from __future__ import annotations

from typing import Any, Callable

from radiate.gp.op import Op

from .handlers import EventHandler
from .dtype import DataType, DataTypeClass


type Subscriber = (
    Callable[[Any], None]
    | list[Callable[[Any], None]]
    | EventHandler
    | list[EventHandler]
)

type NodeValues = list[Op] | Op | list[str] | str

type RdDataType = DataType | DataTypeClass
