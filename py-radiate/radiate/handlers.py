import abc
from typing import Any
from radiate.radiate import PySubscriber


class EventType:
    """
    Enum-like class for event types.
    """

    ALL = "all"
    START = "on_start"
    STOP = "on_stop"
    EPOCH_START = "on_epoch_start"
    EPOCH_COMPLETE = "on_epoch_complete"
    STEP_START = "on_step_start"
    STEP_COMPLETE = "on_step_complete"
    ENGINE_IMPROVEMENT = "on_engine_improvement"


class EventHandler(abc.ABC):
    """
    Base class for event handlers.
    """

    def __init__(self, event_type: EventType = EventType.ALL):
        """
        Initialize the event handler.
        :param event_type: Type of the event to handle.
        """
        self.event_type = event_type if event_type != EventType.ALL else None
        self._py_handler = PySubscriber(self.on_event, self.event_type)

    def __call__(self, event: Any) -> None:
        """
        Call the event handler.
        :param event: The event to handle.
        """
        self.on_event(event)

    @abc.abstractmethod
    def on_event(self, event: Any) -> None:
        """
        Handle the event.
        """
        pass
