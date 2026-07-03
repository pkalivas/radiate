from __future__ import annotations

from pathlib import Path

from radiate.radiate import PyEngineRunOption

from .._bridge.wrapper import RsObject
from .._typing import Checkpoint


class RunParam(RsObject):
    def __init__(self, option: PyEngineRunOption):
        super().__init__(option)


class LogParam(RunParam):
    def __init__(self, enable: bool = False):
        option = PyEngineRunOption.log(enable)
        super().__init__(option)


class CheckpointParam(RunParam):
    def __init__(
        self,
        interval: int = 0,
        path: str | Path = "./checkpoints",
        file_type: str = "pkl",
    ):
        if interval < 0:
            raise ValueError("Checkpoint interval must be a non-negative integer.")

        if not isinstance(path, (str, Path)):
            raise ValueError("Checkpoint path must be a string or Path object.")

        option = PyEngineRunOption.checkpoint(interval, str(path), file_type)
        super().__init__(option)


class UiParam(RunParam):
    def __init__(self):
        option = PyEngineRunOption.ui()
        super().__init__(option)


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
