from __future__ import annotations

from collections.abc import Callable, Sequence
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

if TYPE_CHECKING:
    from .dsl.dtype import DataType, DataTypeClass
    from .dsl.expr import Expr
    from .dsl.loss import LossType, LossTypeClass
    from .engine.handlers import EventHandler
    from .engine.option import CheckpointParam

type FileType = Literal["pkl", "json"]

type AtLeastOne[T] = T | Sequence[T]

type RdDataType = DataType | DataTypeClass

type RdLossType = LossType | LossTypeClass

type Subscriber = AtLeastOne[Callable[[Any], None]] | AtLeastOne[EventHandler]

type OperatorRate = float | Expr

type Checkpoint = (
    bool | str | Path | tuple[int, str | Path, FileType | None] | CheckpointParam
)
