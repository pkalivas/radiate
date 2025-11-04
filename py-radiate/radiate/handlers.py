import abc
from enum import Enum
from typing import Any, Callable
from radiate.radiate import PySubscriber, PyEngineEvent
from radiate.wrapper import PyObject


class EventType(Enum):
    ALL = "all"
    START = "start_event"
    STOP = "stop_event"
    EPOCH_START = "epoch_start_event"
    EPOCH_COMPLETE = "epoch_complete_event"
    ENGINE_IMPROVEMENT = "engine_improvement_event"


class EngineEvent(PyObject[PyEngineEvent]):
    """
    EngineEvent class that wraps around the PyEngineEvent class.
    This class provides a simple interface to access the value of the event.
    """

    def __repr__(self):
        return f"<EngineEvent>{self.__backend__().__repr__()}"

    def __str__(self):
        return f"<EngineEvent>{self.__backend__().__str__()}"
    
    def index(self) -> int | None:
        """
        Get the index of the event.
        :return: The index of the event.
        """
        return self.__backend__().index()
    
    def event_type(self) -> EventType:
        """
        Get the type of the event.
        :return: The type of the event.
        """
        event_type_str = self.__backend__().event_type()
        if event_type_str == "start_event":
            return EventType.START
        elif event_type_str == "stop_event":    
            return EventType.STOP
        elif event_type_str == "epoch_start_event":
            return EventType.EPOCH_START
        elif event_type_str == "epoch_complete_event":
            return EventType.EPOCH_COMPLETE
        elif event_type_str == "engine_improvement_event":
            return EventType.ENGINE_IMPROVEMENT
        else:
            return '<EventType: unknown>'
        
    
    def values(self) -> dict[str, Any]:
        """
        Get all values of the event.
        :return: A dictionary of all values of the event.
        """
        return self.__backend__().values()

    def score(self) -> float | list[float] | None:
        """
        Get the score of the event.
        :return: The score of the event.
        """
        return self.__backend__().score()
    
    def value(self) -> Any:
        """
        Get the value of the event.
        :return: The value of the event.
        """
        return self.__backend__().value()
    
    def metrics(self) -> dict[str, Any] | None:
        """
        Get the metrics of the event.
        :return: The metrics of the event.
        """
        return self.__backend__().metrics()


class EventHandler(abc.ABC):
    """
    Base class for event handlers.
    """

    def __init__(self, event_type: EventType = EventType.ALL):
        """
        Initialize the event handler.
        :param event_type: Type of the event to handle.
        """
        self._py_handler = PySubscriber(
            lambda event: self.on_event(EngineEvent.from_rust(event)), event_type.value
        )

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
