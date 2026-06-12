import abc
from enum import Enum
from typing import Any, Callable

from radiate.radiate import PySubscriber

from radiate._bridge.wrapper import RsObject

from .metrics import MetricSet


class EventType(Enum):
    ALL = "all"
    START = "start_event"
    STOP = "stop_event"
    EPOCH_START = "epoch_start_event"
    EPOCH_COMPLETE = "epoch_complete_event"
    ENGINE_IMPROVEMENT = "engine_improvement_event"


class EngineEvent(RsObject):
    """
    EngineEvent class that wraps around the PyEngineEvent class.
    This class provides a simple interface to access the value of the event.
    """

    def __repr__(self):
        return f"<EngineEvent>{self.__backend__().__repr__()}"

    def __str__(self):
        return self.__repr__()

    def index(self) -> int:
        """
        Get the index of the event.
        :return: The index of the event.
        """
        index = self.__backend__().index()
        return index if index is not None else 0

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
            raise ValueError(f"Unknown event type: {event_type_str}")

    def score(self) -> float | list[float] | None:
        """
        Get the score of the event.
        :return: The score of the event.
        """
        return self.try_get_cache("score_cache", lambda: self.__backend__().score())

    def value(self) -> Any:
        """
        Get the value of the event.
        :return: The value of the event.
        """
        return self.try_get_cache("value_cache", lambda: self.__backend__().best())

    def metrics(self) -> MetricSet:
        """
        Get the metrics of the event.
        :return: The metrics of the event.
        """

        def _acquire_metrics():
            metrics = self.__backend__().metrics()
            if metrics is None:
                return MetricSet()
            return MetricSet.from_rust(metrics)

        return self.try_get_cache("metrics_cache", _acquire_metrics)

    def objective(self) -> list[str] | None:
        """
        Get the objective of the event.
        :return: The objective of the event.
        """
        return self.try_get_cache(
            "objective_cache", lambda: self.__backend__().objective()
        )


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
        from radiate._dependancies import _POLARS_AVAILABLE

        if not _POLARS_AVAILABLE:
            raise ImportError(
                "Polars is not available. Please install it to use this feature."
            )
        from radiate._dependancies import polars as pl

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
        from radiate._dependancies import _PANDAS_AVAILABLE

        if not _PANDAS_AVAILABLE:
            raise ImportError(
                "Pandas is not available. Please install it to use this feature."
            )
        from radiate._dependancies import pandas as pd

        return pd.DataFrame(
            [
                m.to_dict()
                for metric_set in self.metric_history
                for m in metric_set.values()
            ]
        )

    def plot(self, *names: str):
        from radiate._dependancies import _MATPLOTLIB_AVAILABLE

        if not _MATPLOTLIB_AVAILABLE:
            raise ImportError(
                "Matplotlib is not available. Please install it to use this feature."
            )

        from radiate._dependancies import matplotlib as plt

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
