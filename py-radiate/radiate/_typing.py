from __future__ import annotations

from collections.abc import Sequence, Callable
from typing import Any, TYPE_CHECKING


if TYPE_CHECKING:
    from ._dependancies import numpy as np
    from .engine.handlers import EventHandler
    from .dtype import DataType, DataTypeClass
    from radiate.gp import Graph, Tree


type AtLeastOne[T] = T | Sequence[T]

type Primitive = int | float | bool | str

type RdDataType = DataType | DataTypeClass

type Subscriber = AtLeastOne[Callable[[Any], None]] | AtLeastOne[EventHandler]

type MatrixDecoding[T] = Sequence[Sequence[T]] | Sequence["np.ndarray"]
type GpDecoding = Graph | Tree
