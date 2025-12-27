from __future__ import annotations

import enum
from datetime import timedelta

from radiate.radiate import PyMetricSet, PyMetric, PyTagKind
from radiate.wrapper import PyObject
from radiate.dependancies import _PANDAS_AVAILABLE, _POLARS_AVAILABLE


class Tag(enum.Enum):
    """Enumeration of metric tag kinds."""

    SELECTOR = PyTagKind.Selector
    ALTERER = PyTagKind.Alterer
    MUTATOR = PyTagKind.Mutator
    CROSSOVER = PyTagKind.Crossover
    SPECIES = PyTagKind.Species
    FAILURE = PyTagKind.Failure
    AGE = PyTagKind.Age
    FRONT = PyTagKind.Front
    DERIVED = PyTagKind.Derived
    OTHER = PyTagKind.Other
    STATISTIC = PyTagKind.Statistic
    TIME = PyTagKind.Time
    DISTRIBUTION = PyTagKind.Distribution
    SCORE = PyTagKind.Score

    def __repr__(self) -> str:
        return f"Tag.{self.name}"


class MetricSet(PyObject[PyMetricSet]):
    def __repr__(self):
        return self.__backend__().__repr__()

    def __getitem__(self, key: str) -> "Metric":
        return Metric.from_rust(self.__backend__().__getitem__(key))

    def __dict__(self) -> dict:
        return self.__backend__().__dict__()

    def __len__(self) -> int:
        return self.__backend__().__len__()

    def __contains__(self, item: str) -> bool:
        return self.__backend__().__contains__(item)

    def dashboard(self) -> str:
        return self.__backend__().dashboard()

    def keys(self) -> list[str]:
        return self.__backend__().keys()

    def values(self) -> list[Metric]:
        return [Metric.from_rust(m) for m in self.__backend__().values()]

    def values_by_tag(self, tag: Tag) -> list[Metric]:
        return [
            Metric.from_rust(m) for m in self.__backend__().values_by_tag(tag.value)
        ]

    def to_polars(self):
        if not _POLARS_AVAILABLE:
            raise ImportError(
                "Polars is not available. Please install it to use this feature."
            )
        return self.__backend__().to_polars()

    def to_pandas(self):
        if not _PANDAS_AVAILABLE:
            raise ImportError(
                "Pandas is not available. Please install it to use this feature."
            )
        return self.__backend__().to_pandas()


class Metric(PyObject[PyMetric]):
    def __repr__(self) -> str:
        return self.__backend__().__repr__()

    def __dict__(self) -> dict:
        return self.__backend__().__dict__()

    def name(self) -> str:
        return self.__backend__().name

    # --- value stats ---
    def value_last(self) -> float:
        return self.__backend__().value_last

    def mean(self) -> float | None:
        return self.__backend__().value_mean

    def sum(self) -> float | None:
        return self.__backend__().value_sum

    def stddev(self) -> float | None:
        return self.__backend__().value_stddev

    def variance(self) -> float | None:
        return self.__backend__().value_variance

    def skew(self) -> float | None:
        return self.__backend__().value_skewness

    def min(self) -> float | None:
        return self.__backend__().value_min

    def max(self) -> float | None:
        return self.__backend__().value_max

    def count(self) -> int:
        return self.__backend__().value_count

    # --- time stats ---
    def time_last(self) -> timedelta | None:
        last_time = self.__backend__().time_last
        if last_time is None:
            return None
        return timedelta(seconds=last_time)

    def time_sum(self) -> timedelta | None:
        sum_time = self.__backend__().time_sum
        if sum_time is None:
            return None
        return timedelta(seconds=sum_time)

    def time_mean(self) -> timedelta | None:
        mean_time = self.__backend__().time_mean
        if mean_time is None:
            return None
        return timedelta(seconds=mean_time)

    def time_stddev(self) -> timedelta | None:
        stddev_time = self.__backend__().time_stddev
        if stddev_time is None:
            return None
        return timedelta(seconds=stddev_time)

    def time_variance(self) -> timedelta | None:
        variance_time = self.__backend__().time_variance
        if variance_time is None:
            return None
        return timedelta(seconds=variance_time)

    def time_min(self) -> timedelta | None:
        time_min = self.__backend__().time_min
        if time_min is None:
            return None
        return timedelta(seconds=time_min)

    def time_max(self) -> timedelta | None:
        time_max = self.__backend__().time_max
        if time_max is None:
            return None
        return timedelta(seconds=time_max)
