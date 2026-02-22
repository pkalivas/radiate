from __future__ import annotations

from radiate.radiate import PyEngineRunOption
from radiate._bridge.wrapper import RsObject


class RunOption(RsObject):
    def __init__(self, option: PyEngineRunOption):
        super().__init__(option)


class EngineLog(RunOption):
    def __init__(self, enable: bool = False):
        option = PyEngineRunOption.log(enable)
        super().__init__(option)


class EngineCheckpoint(RunOption):
    def __init__(self, interval: int = 0, path: str = "./checkpoints"):
        option = PyEngineRunOption.checkpoint(interval, path)
        super().__init__(option)


class EngineUi(RunOption):
    def __init__(self):
        option = PyEngineRunOption.ui()
        super().__init__(option)
