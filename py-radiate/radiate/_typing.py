from __future__ import annotations

from collections.abc import Sequence, Callable
from pathlib import Path
from typing import Any, TYPE_CHECKING, Literal


if TYPE_CHECKING:
    from .expr import Expr
    from .operators.rate import Rate
    from .engine.handlers import EventHandler
    from .dtype import DataType, DataTypeClass
    from .fitness.loss import LossType, LossTypeClass
    from .engine.option import CheckpointParam

type FileType = Literal["pkl", "json"]

type AtLeastOne[T] = T | Sequence[T]

type RdDataType = DataType | DataTypeClass

type RdLossType = LossType | LossTypeClass

type Subscriber = AtLeastOne[Callable[[Any], None]] | AtLeastOne[EventHandler]

type OperatorRate = float | Rate | Expr

type Checkpoint = (
    bool | str | Path | tuple[int, str | Path, FileType | None] | CheckpointParam
)
