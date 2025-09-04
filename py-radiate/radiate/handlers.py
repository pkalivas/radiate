import abc
from enum import Enum
from typing import Any, Callable
from radiate.radiate import PySubscriber


class EventType(Enum):
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
        self._py_handler = PySubscriber(self.on_event, event_type.value)

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


class CallableEventHandler(EventHandler):
    def __init__(
        self, func: Callable[[Any], None], event_type: EventType = EventType.ALL
    ):
        super().__init__(event_type)
        self.func = func

    def on_event(self, event: Any) -> None:
        self.func(event)
