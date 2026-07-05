from .._bridge.wrapper import RsObject
from .._rd import PyEngineControl


class EngineControl(RsObject):
    def __init__(self, control: PyEngineControl):
        super().__init__(control)

    def pause(self) -> None:
        """Pause the engine's execution."""
        self.__backend__().pause()

    def resume(self) -> None:
        """Resume the engine's execution."""
        self.__backend__().resume()
