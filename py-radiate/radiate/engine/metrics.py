from __future__ import annotations

import enum
from datetime import timedelta
from typing import Any, Iterator

from radiate.radiate import PyMetricSet

from .._bridge.wrapper import RsObject


class Tag(enum.Enum):
    """Enumeration of metric tag kinds."""

    SELECTOR = "selector"
    ALTERER = "alterer"
    MUTATOR = "mutator"
    CROSSOVER = "crossover"
    SPECIES = "species"
    FAILURE = "failure"
    AGE = "age"
    FRONT = "front"
    DERIVED = "derived"
    OTHER = "other"
    STATISTIC = "statistic"
    TIME = "time"
    DISTRIBUTION = "distribution"
    SCORE = "score"
    RATE = "rate"
    STEP = "step"
    EXPR = "expr"

    def __repr__(self) -> str:
        return f"Tag.{self.name}"


tag_map = {
    "py": {
        "selector": Tag.SELECTOR,
        "alterer": Tag.ALTERER,
        "mutator": Tag.MUTATOR,
        "crossover": Tag.CROSSOVER,
        "species": Tag.SPECIES,
        "failure": Tag.FAILURE,
        "age": Tag.AGE,
        "front": Tag.FRONT,
        "derived": Tag.DERIVED,
        "other": Tag.OTHER,
        "statistic": Tag.STATISTIC,
        "time": Tag.TIME,
        "distribution": Tag.DISTRIBUTION,
        "score": Tag.SCORE,
        "rate": Tag.RATE,
        "step": Tag.STEP,
        "expr": Tag.EXPR,
    },
    "rs": {
        Tag.SELECTOR: "selector",
        Tag.ALTERER: "alterer",
        Tag.MUTATOR: "mutator",
        Tag.CROSSOVER: "crossover",
        Tag.SPECIES: "species",
        Tag.FAILURE: "failure",
        Tag.AGE: "age",
        Tag.FRONT: "front",
        Tag.DERIVED: "derived",
        Tag.OTHER: "other",
        Tag.STATISTIC: "statistic",
        Tag.TIME: "time",
        Tag.DISTRIBUTION: "distribution",
        Tag.SCORE: "score",
        Tag.RATE: "rate",
        Tag.STEP: "step",
        Tag.EXPR: "expr",
    },
}


class MetricSet(RsObject):
    def __init__(self, values: dict[str, Any] | None = None, **kwargs):
        update = values.copy() if values else {}
        update.update(kwargs)
        super().__init__(PyMetricSet(update))

    def __repr__(self):
        return self.__backend__().__repr__()

    def __getitem__(self, key: str) -> "Metric":
        return Metric.from_rust(self.__backend__().__getitem__(key))

    def __dict__(self) -> dict[str, Metric]:  # type: ignore
        return {
            key: Metric.from_rust(m) for key, m in self.__backend__().__dict__().items()
        }

    def __len__(self) -> int:
        return self.__backend__().__len__()

    def __contains__(self, item: str) -> bool:
        return self.__backend__().__contains__(item)

    def __iter__(self) -> "Iterator[Metric]":
        for key in self.keys():
            yield self[key]

    def dashboard(self) -> str:
        return self.__backend__().dashboard()

    def keys(self) -> list[str]:
        return self.__backend__().keys()

    def values(self) -> list[Metric]:
        return [Metric.from_rust(m) for m in self.__backend__().values()]

    def values_by_tag(self, tag: Tag) -> list[Metric]:
        return [
            Metric.from_rust(m)
            for m in self.__backend__().values_by_tag(tag_map["rs"][tag])
        ]

    def to_polars(self, lazy: bool = False):
        from .._dependancies import _POLARS_AVAILABLE

        if not _POLARS_AVAILABLE:
            raise ImportError(
                "Polars is not available. Please install it to use this feature."
            )
        return self.__backend__().to_polars(lazy=lazy)

    def to_pandas(self):
        from .._dependancies import _PANDAS_AVAILABLE

        if not _PANDAS_AVAILABLE:
            raise ImportError(
                "Pandas is not available. Please install it to use this feature."
            )
        return self.__backend__().to_pandas()

    def upsert(self, name: str, value: Any) -> None:
        """Upsert new metrics into the MetricSet."""
        self.__backend__().upsert(name, value)


class Metric(RsObject):
    def __repr__(self) -> str:
        return self.__backend__().__repr__()

    def __str__(self) -> str:
        return self.__backend__().__repr__()

    def name(self) -> str:
        return self.__backend__().name

    def tags(self) -> list[Tag]:
        return [tag_map["py"][t] for t in self.__backend__().tags]

    def to_dict(self) -> dict[str, Any]:
        return self.__backend__().to_dict()

    def version(self) -> int:
        return self.__backend__().version

    def update_count(self) -> int:
        return self.__backend__().update_count

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
        return self.__backend__().time_last

    def time_sum(self) -> timedelta | None:
        return self.__backend__().time_sum

    def time_mean(self) -> timedelta | None:
        return self.__backend__().time_mean

    def time_stddev(self) -> timedelta | None:
        return self.__backend__().time_stddev

    def time_variance(self) -> timedelta | None:
        return self.__backend__().time_variance

    def time_min(self) -> timedelta | None:
        return self.__backend__().time_min

    def time_max(self) -> timedelta | None:
        return self.__backend__().time_max
