from __future__ import annotations

from collections.abc import Sequence, Callable
from typing import Any, TYPE_CHECKING


if TYPE_CHECKING:
    from .engine.handlers import EventHandler
    from .dtype import DataType, DataTypeClass
    from radiate.fitness.loss import LossType, LossTypeClass


type AtLeastOne[T] = T | Sequence[T]

type RdDataType = DataType | DataTypeClass

type RdLossType = LossType | LossTypeClass

type Subscriber = AtLeastOne[Callable[[Any], None]] | AtLeastOne[EventHandler]
