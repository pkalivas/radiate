from __future__ import annotations

from collections.abc import Sequence, Callable
from typing import Any, TYPE_CHECKING


if TYPE_CHECKING:
    from ._dependancies import numpy as np
    from .engine.handlers import EventHandler
    from .dtype import DataType, DataTypeClass
    from radiate.codec import CodecBase
    from radiate.genome import Gene, Chromosome
else:
    DataType = DataTypeClass = str
    EventHandler = Callable[[Any], None]
    CodecBase = Gene = Chromosome = object

type Vec[T] = Sequence[T]
type Mat[T] = Sequence[Sequence[T]]
type Layout[T] = T | Vec[T] | Mat[T]

type AtLeastOne[T] = T | Vec[T]

type Primitive = int | float | bool | str

type RdDataType = DataType | DataTypeClass

type Subscriber = AtLeastOne[Callable[[Any], None]] | AtLeastOne[EventHandler]

type Decoding[T] = T | Vec[T] | Mat[T] | "np.ndarray"
type Encoding[T] = Layout[Primitive] | Layout[T] | CodecBase[Gene[T], T]
