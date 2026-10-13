from __future__ import annotations

from pathlib import Path

from .._typing import Checkpoint, FileType


class CheckpointParam:
    def __init__(
        self,
        interval: int = 0,
        path: str | Path = "./checkpoints",
        file_type: FileType = "pkl",
    ):
        if interval < 0:
            raise ValueError("Checkpoint interval must be a non-negative integer.")

        if not isinstance(path, (str, Path)):
            raise ValueError("Checkpoint path must be a string or Path object.")

        self.interval = interval
        self.path = path
        self.file_type = file_type


def normalize_checkpoint_params(
    checkpoint: Checkpoint | None,
) -> CheckpointParam | None:
    match checkpoint:
        case False | None:
            return None

        case True:
            return CheckpointParam(interval=250, path="checkpoints")
        case CheckpointParam() as option:
            return option
        case str() | Path() as path:
            return CheckpointParam(interval=250, path=path)
        case (int(interval), str() | Path() as path):
            return CheckpointParam(interval=interval, path=path)
        case (int(interval), str() | Path() as path, str() as file_type):
            return CheckpointParam(interval=interval, path=path, file_type=file_type)
        case _:
            raise TypeError(
                "checkpoint must be False, True, a path, "
                "(interval, path), or CheckpointParam."
            )
