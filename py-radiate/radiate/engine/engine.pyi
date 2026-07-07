from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal, Self, Sequence, overload

from radiate._typing import AtLeastOne, FileType, RdDataType, RdLossType, Subscriber
from radiate.codec.base import CodecBase
from radiate.dsl.dtype import Float32, Int8, Int16, Int32, UInt8, UInt16, UInt32, UInt64
from radiate.dsl.expr import Expr
from radiate.fitness import MSE, FitnessBase
from radiate.genome import Chromosome, Gene, Population
from radiate.gp import Graph, Op, Tree
from radiate.operators import (
    AlterBase,
    Fitness,
)
from radiate.operators.filter import Filter
from radiate.operators.limit import Limit
from radiate.operators.selector import Select

from ..operators.distance import Dist
from .generation import Generation
from .option import CheckpointParam

if TYPE_CHECKING:
    from radiate._dependancies import numpy as np

class Engine[G, T]:
    def __init__(self, codec: CodecBase[G, T], **kwargs: Any) -> None: ...

    # ----------------------------
    # Float engine constructors
    # ----------------------------

    # Scalar
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[float, float]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: Gene[float],
        chromosomes: None = ...,
    ) -> "Engine[float, float]": ...

    # Vector via shape
    @overload
    @staticmethod
    def float(
        shape: int,
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[float, list[float]]": ...
    @overload
    @staticmethod
    def float(
        shape: int,
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: type[Float32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[float, np.typing.NDArray[np.float32]]": ...
    @overload
    @staticmethod
    def float(
        shape: int,
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[float, np.typing.NDArray[np.float64]]": ...

    # Vector via genes
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: Sequence[Gene[float]],
        chromosomes: None = ...,
    ) -> "Engine[float, list[float]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: type[Float32],
        use_numpy: Literal[True],
        genes: Sequence[Gene[float]],
        chromosomes: None = ...,
    ) -> "Engine[float, np.typing.NDArray[np.float32]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: Sequence[Gene[float]],
        chromosomes: None = ...,
    ) -> "Engine[float, np.typing.NDArray[np.float64]]": ...

    # Vector via chromosome (dtype narrowing deferred)
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Chromosome[float],
    ) -> "Engine[float, list[float]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Chromosome[float],
    ) -> "Engine[float, np.ndarray]": ...

    # Matrix via shape
    @overload
    @staticmethod
    def float(
        shape: Sequence[int],
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[float, list[list[float]]]": ...
    @overload
    @staticmethod
    def float(
        shape: Sequence[int],
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: type[Float32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[float, list[np.typing.NDArray[np.float32]]]": ...
    @overload
    @staticmethod
    def float(
        shape: Sequence[int],
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[float, list[np.typing.NDArray[np.float64]]]": ...

    # Matrix via chromosomes (dtype narrowing deferred)
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[float]],
    ) -> "Engine[float, list[list[float]]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Sequence[Chromosome[float]],
    ) -> "Engine[float, list[np.ndarray]]": ...
    @staticmethod
    def float(
        shape: AtLeastOne[int] | None = None,
        *,
        init_range: tuple[float, float] | None = (0, 1.0),
        bounds: tuple[float, float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = False,
        genes: AtLeastOne[Gene[float]] | None = None,
        chromosomes: AtLeastOne[Chromosome[float]] | None = None,
    ) -> "Engine[float, Any]": ...

    # ----------------------------
    # Int engine constructors
    # ----------------------------

    # Scalar
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, int]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: Gene[int],
        chromosomes: None = ...,
    ) -> "Engine[int, int]": ...

    # Vector via shape
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[int]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int8]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int16]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int32]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint8]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint16]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint32]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt64],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint64]]": ...
    @overload
    @staticmethod
    def int(
        shape: int,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int64]]": ...

    # Vector via genes
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, list[int]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int8],
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int8]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int16],
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int16]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int32],
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int32]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt8],
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint8]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt16],
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint16]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt32],
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint32]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt64],
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.uint64]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: Sequence[Gene[int]],
        chromosomes: None = ...,
    ) -> "Engine[int, np.typing.NDArray[np.int64]]": ...

    # Vector via chromosome (dtype narrowing deferred)
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Chromosome[int],
    ) -> "Engine[int, list[int]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Chromosome[int],
    ) -> "Engine[int, np.ndarray]": ...

    # Matrix via shape
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[list[int]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.int8]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.int16]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[Int32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.int32]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.uint8]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.uint16]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.uint32]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: type[UInt64],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.uint64]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[int],
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[int, list[np.typing.NDArray[np.int64]]]": ...

    # Matrix via chromosomes (dtype narrowing deferred)
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[int]],
    ) -> "Engine[int, list[list[int]]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Sequence[Chromosome[int]],
    ) -> "Engine[int, list[np.ndarray]]": ...
    @staticmethod
    def int(
        shape: AtLeastOne[int] | None = None,
        *,
        init_range: tuple[int, int] | None = (0, 100),
        bounds: tuple[int, int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = False,
        genes: AtLeastOne[Gene[int]] | None = None,
        chromosomes: AtLeastOne[Chromosome[int]] | None = None,
    ) -> "Engine[int, Any]": ...

    # ----------------------------
    # Char engine constructors
    # ----------------------------

    @overload
    @staticmethod
    def char(
        shape: int,
        *,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, list[str]]": ...
    @overload
    @staticmethod
    def char(
        shape: Sequence[int],
        *,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, list[list[str]]]": ...
    @staticmethod
    def char(
        shape: AtLeastOne[int] = 1,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, Any]": ...

    # ----------------------------
    # Bit engine constructors
    # ----------------------------

    @overload
    @staticmethod
    def bit(
        shape: int,
        use_numpy: Literal[False] = False,
    ) -> "Engine[bool, list[bool]]": ...
    @overload
    @staticmethod
    def bit(
        shape: int,
        use_numpy: Literal[True] = True,
    ) -> "Engine[bool, np.ndarray]": ...
    @overload
    @staticmethod
    def bit(
        shape: Sequence[int],
        use_numpy: Literal[False] = False,
    ) -> "Engine[bool, list[list[bool]]]": ...
    @overload
    @staticmethod
    def bit(
        shape: Sequence[int],
        use_numpy: Literal[True] = True,
    ) -> "Engine[bool, list[np.ndarray]]": ...
    @staticmethod
    def bit(
        shape: AtLeastOne[int] = 1,
        use_numpy: bool = False,
    ) -> "Engine[bool, Any]": ...

    # ----------------------------
    # Other constructors
    # ----------------------------

    @staticmethod
    def permutation[P](items: list[P]) -> "Engine[P, list[P]]": ...
    @staticmethod
    def graph(
        shape: tuple[int, int],
        vertex: Op | list[Op] | None = None,
        edge: Op | list[Op] | None = None,
        output: Op | list[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
        max_nodes: int | None = None,
        graph_type: str = "directed",
    ) -> "Engine[Op, Graph]": ...
    @staticmethod
    def tree(
        shape: tuple[int, int] = (1, 1),
        min_depth: int = 3,
        max_size: int = 30,
        vertex: Op | list[Op] | None = None,
        leaf: Op | list[Op] | None = None,
        root: Op | list[Op] | None = None,
        values: dict[str, AtLeastOne[Op]] | None = None,
    ) -> "Engine[Op, Tree]": ...

    # ----------------------------
    # Iteration / execution
    # ----------------------------
    def __iter__(self) -> Engine[G, T]: ...
    def __next__(self) -> Generation[G, T]: ...
    def run(
        self,
        log: bool = False,
        ui: bool = False,
        checkpoint: str
        | Path
        | tuple[int, str | Path, FileType | None]
        | CheckpointParam
        | None = None,
    ) -> Generation[G, T]: ...
    def nth(self, n: int) -> Generation[G, T]: ...

    # ----------------------------
    # Fluent configuration methods
    # ----------------------------

    def fitness(self, fitness_func: Callable[[T], Any] | Fitness[T]) -> Self: ...
    def regression(
        self,
        features: Any,
        targets: Any | None = None,
        *,
        target_cols: str | list[str] | None = None,
        feature_cols: list[str] | None = None,
        loss: RdLossType | None = MSE,
        batch: bool = False,
    ) -> Self: ...
    def select(
        self,
        offspring: Select | None = None,
        survivor: Select | None = None,
        frac: float | None = None,
    ) -> Self: ...
    def alters(self, *alters: AlterBase) -> Self: ...
    def diversity(
        self,
        diversity: Dist,
        threshold: Expr | float = 0.5,
        target: int | None = None,
    ) -> Self: ...
    def limit(self, *limits: Limit | Expr) -> Self: ...
    def filter(self, *filters: Filter) -> Self: ...
    def size(self, size: int) -> Self: ...
    def age(
        self,
        max_phenotype_age: int | None = None,
        max_species_age: int | None = None,
    ) -> Self: ...
    def population(self, population: Population[G]) -> Self: ...
    def minimizing(self) -> Self: ...
    def maximizing(self) -> Self: ...
    def objective(
        self,
        *obj: str,
        front_range: tuple[int, int] | None = None,
    ) -> Self: ...
    def front_range(self, min: int, max: int) -> Self: ...
    def parallel(self, num_workers: int | None = None) -> Self: ...
    def subscribe(self, *event_handler: Subscriber) -> Self: ...
    def generation(self, generation: Generation[G, T] | None) -> Self: ...
    def load_checkpoint(
        self, path: str | Path, ignore_not_found: bool = False
    ) -> Self: ...
    def metrics(
        self, named_metrics: dict[str, Expr] | None = None, **kwargs
    ) -> Self: ...
