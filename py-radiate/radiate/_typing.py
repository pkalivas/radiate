from typing import Any, Callable, List, TypeAlias, Union

from .handlers import EventHandler

Subscriber: TypeAlias = Union[
    Callable[[Any], None], List[Callable[[Any], None]], EventHandler, List[EventHandler]
]