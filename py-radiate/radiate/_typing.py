from typing import Any, Callable, List, Union

from .handlers import EventHandler

type Subscriber = Union[
    Callable[[Any], None], List[Callable[[Any], None]], EventHandler, List[EventHandler]
]
