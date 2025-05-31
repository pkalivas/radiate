from typing import Callable, Any

from radiate.radiate import PyFunc

class EventHandler:
    """
    Base class for event handlers.
    """

    def __init__(self, handler: Callable[[Any], None]):
        """
        Initialize the event handler.
        """
        self.handler = PyFunc(handler)


class LogHandler(EventHandler):
    """
    Handler for log events.
    """

    def __init__(self):
        """
        Initialize the log handler.
        """
        super().__init__(self.handle_event)

    def handle_event(self, event: Any) -> None:
        """
        Handle a log event.
        """
        print(f"Log event: {event}")
