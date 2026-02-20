from __future__ import annotations

from collections.abc import Sequence, Callable
from typing import Any, TYPE_CHECKING


if TYPE_CHECKING:
    from ._dependancies import numpy as np
    from .engine.handlers import EventHandler
    from .dtype import DataType, DataTypeClass
    from radiate.codec import CodecBase
    from radiate.genome import Gene
    from radiate.gp import Graph, Tree


type AtLeastOne[T] = T | Sequence[T]

type Primitive = int | float | bool | str

type RdDataType = DataType | DataTypeClass

type Subscriber = AtLeastOne[Callable[[Any], None]] | AtLeastOne[EventHandler]

type ScalarDecoding[T] = T
type VectorDecoding[T] = Sequence[T] | "np.ndarray"
type MatrixDecoding[T] = Sequence[Sequence[T]] | Sequence["np.ndarray"]
type GpDecoding = Graph | Tree
type Decoding[T] = (
    ScalarDecoding[T] | VectorDecoding[T] | MatrixDecoding[T] | GpDecoding
)


type Encoding[T] = (
    "Gene[T]"
    | Sequence["Gene[T]"]
    | Sequence[Sequence["Gene[T]"]]
    | "CodecBase[T, Decoding[T]]"
)
