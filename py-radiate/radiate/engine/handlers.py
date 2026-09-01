import abc
from enum import Enum
from typing import Any, Callable

from radiate.radiate import PySubscriber

from .._bridge import RsObject
from .metrics import MetricSet


class EventType(Enum):
    ALL = "all"
    START = "start_event"
    STOP = "stop_event"
    EPOCH_START = "epoch_start_event"
    EPOCH_COMPLETE = "epoch_complete_event"
    ENGINE_IMPROVEMENT = "engine_improvement_event"
    LIMIT_TRIGGERED = "limit_triggered_event"
    LOG = "log_event"
    CHECKPOINT_SAVED = "checkpoint_saved_event"


class EngineEvent(RsObject):
    """
    EngineEvent class that wraps around the PyEngineEvent class.
    This class provides a simple interface to access the value of the event.
    """

    event_type: EventType
    index: int
    data: Any

    def __init__(self, event_type: str, index: int | None, data: Any):
        self.event_type = EventType(event_type)
        self.index = index if index is not None else 0
        self.data = data

    def __repr__(self):
        return f"<EngineEvent type={self.event_type}, index={self.index}, data={self.data}>"

    def __str__(self):
        return self.__repr__()

    def score(self) -> list[float] | None:
        """
        Get the score of the event.
        :return: The score of the event.
        """
        return self.data.get("score", None)

    def value(self) -> Any:
        """
        Get the value of the event.
        :return: The value of the event.
        """
        return self.data.get("best", None)

    def metrics(self) -> MetricSet:
        """
        Get the metrics of the event.
        :return: The metrics of the event.
        """

        metrics = self.data.get("metrics", None)
        if metrics is None:
            return MetricSet()
        return MetricSet.from_rust(metrics)

    def objective(self) -> list[str] | None:
        """
        Get the objective of the event.
        :return: The objective of the event.
        """
        return self.data.get("objective", None)

    def limit(self) -> str | None:
        """
        Get the limit of the event.
        :return: The limit of the event.
        """
        return self.data.get("limit", None)


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
            lambda event: self.on_event(event), event_type.value
        )

    def __call__(self, event: "EngineEvent") -> None:
        """
        Call the event handler.
        :param event: The event to handle.
        """
        self.on_event(event)

    @abc.abstractmethod
    def on_event(self, event: "EngineEvent") -> None:
        """
        Handle the event.
        """
        pass


class CallableEventHandler(EventHandler):
    def __init__(
        self,
        func: Callable[["EngineEvent"], None],
        event_type: EventType = EventType.ALL,
    ):
        super().__init__(event_type)
        self.func = func

    def on_event(self, event: "EngineEvent") -> None:
        self.func(event)


class MetricCollector(EventHandler):
    def __init__(self):
        super().__init__(EventType.EPOCH_COMPLETE)
        self.metric_history: list[MetricSet] = []

    def on_event(self, event: "EngineEvent") -> None:
        metrics = event.metrics()
        self.metric_history.append(metrics)

    def to_polars(self, lazy: bool = False):
        from .._dependancies import _POLARS_AVAILABLE

        if not _POLARS_AVAILABLE:
            raise ImportError(
                "Polars is not available. Please install it to use this feature."
            )
        from .._dependancies import polars as pl

        if lazy:
            return pl.LazyFrame(
                [
                    m.to_dict()
                    for metric_set in self.metric_history
                    for m in metric_set.values()
                ]
            )

        return pl.DataFrame(
            [
                m.to_dict()
                for metric_set in self.metric_history
                for m in metric_set.values()
            ]
        )

    def to_pandas(self):
        from .._dependancies import _PANDAS_AVAILABLE

        if not _PANDAS_AVAILABLE:
            raise ImportError(
                "Pandas is not available. Please install it to use this feature."
            )
        from .._dependancies import pandas as pd

        return pd.DataFrame(
            [
                m.to_dict()
                for metric_set in self.metric_history
                for m in metric_set.values()
            ]
        )

    def plot(self, *names: str):
        from .._dependancies import _MATPLOTLIB_AVAILABLE

        if not _MATPLOTLIB_AVAILABLE:
            raise ImportError(
                "Matplotlib is not available. Please install it to use this feature."
            )

        from .._dependancies import matplotlib as plt

        vals = {name: [] for name in names}
        for metric_set in self.metric_history:
            for name in names:
                metric = metric_set[name]
                vals[name].append(metric.value_last())

        x = list(range(max(len(v) for v in vals.values())))
        for name, scores in vals.items():
            plt.plot(x, scores, label=name)

        plt.xlabel("Epoch")
        plt.ylabel("Value")
        plt.title("Metrics over Epochs")
        plt.grid(True)
        plt.legend()
        plt.show()


def on_epoch(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for the EPOCH_COMPLETE event.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.EPOCH_COMPLETE)


def on_start(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for the START event.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.START)


def on_stop(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for the STOP event.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.STOP)


def on_improvement(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for the ENGINE_IMPROVEMENT event.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.ENGINE_IMPROVEMENT)


def on_limit_triggered(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for the LIMIT_TRIGGERED event.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.LIMIT_TRIGGERED)


def on_log(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for the LOG event.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.LOG)


def on_checkpoint_saved(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for the CHECKPOINT_SAVED event.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.CHECKPOINT_SAVED)


def on_event(func: Callable[["EngineEvent"], None]) -> CallableEventHandler:
    """
    Decorator to register a function as an event handler for all events.
    :param func: The function to register as an event handler.
    :return: A CallableEventHandler instance.
    """
    return CallableEventHandler(func, EventType.ALL)
