from __future__ import annotations

from collections.abc import Callable, Sequence
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

if TYPE_CHECKING:
    from .dtype import DataType, DataTypeClass
    from .engine.handlers import EventHandler
    from .engine.option import CheckpointParam
    from .expr import Expr
    from .fitness.loss import LossType, LossTypeClass

type FileType = Literal["pkl", "json"]

type AtLeastOne[T] = T | Sequence[T]

type RdDataType = DataType | DataTypeClass

type RdLossType = LossType | LossTypeClass

type Subscriber = AtLeastOne[Callable[[Any], None]] | AtLeastOne[EventHandler]

type OperatorRate = float | Expr

type Checkpoint = (
    bool | str | Path | tuple[int, str | Path, FileType | None] | CheckpointParam
)
