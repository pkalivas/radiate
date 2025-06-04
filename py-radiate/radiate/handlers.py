import abc
from typing import Callable, Any

from radiate.radiate import PySubscriber


class EventHandler(abc.ABC):
    """
    Base class for event handlers.
    """

    def __call__(self, event: Any) -> None:
        """
        Call the handler with the event.
        """
        self.on_event(event)

    @abc.abstractmethod
    def on_event(self, event: Any) -> None:
        """
        Handle the event.
        """
        pass


class OnEpochCompleteHandler(EventHandler):
    """
    Handler for the end of an epoch.
    """

    def __init__(self, callback: Callable[[Any], None]):
        """
        Initialize the handler with a callback function.
        :param callback: Function to call when the epoch is complete.
        """
        super().__init__()
        self.callback = callback

    def on_event(self, event: Any) -> None:
        """
        Call the callback function with the event.
        :param event: The event data.
        """
        self.callback(event)
