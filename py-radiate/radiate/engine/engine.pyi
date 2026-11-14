from __future__ import annotations

import builtins
from collections.abc import Callable
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal, Self, Sequence, overload

from radiate._typing import AtLeastOne, FileType, RdDataType, RdLossType, Subscriber
from radiate.codec.base import CodecBase
from radiate.dsl.dtype import (
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
)
from radiate.dsl.expr import Expr
from radiate.dsl.loss import MSE
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
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, builtins.float]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: Gene[builtins.float],
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, builtins.float]": ...

    # Vector via shape
    @overload
    @staticmethod
    def float(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, list[builtins.float]]": ...
    @overload
    @staticmethod
    def float(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: type[Float32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, np.typing.NDArray[np.float32]]": ...
    @overload
    @staticmethod
    def float(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, np.typing.NDArray[np.float64]]": ...

    # Vector via genes
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: Sequence[Gene[builtins.float]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, list[builtins.float]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: type[Float32],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.float]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, np.typing.NDArray[np.float32]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.float]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, np.typing.NDArray[np.float64]]": ...

    # Vector via chromosome (dtype narrowing deferred)
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Chromosome[builtins.float],
    ) -> "Engine[builtins.float, list[builtins.float]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Chromosome[builtins.float],
    ) -> "Engine[builtins.float, np.ndarray]": ...

    # Matrix via shape
    @overload
    @staticmethod
    def float(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, list[list[builtins.float]]]": ...
    @overload
    @staticmethod
    def float(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: type[Float32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, list[np.typing.NDArray[np.float32]]]": ...
    @overload
    @staticmethod
    def float(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.float, list[np.typing.NDArray[np.float64]]]": ...

    # Matrix via chromosomes (dtype narrowing deferred)
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[builtins.float]],
    ) -> "Engine[builtins.float, list[list[builtins.float]]]": ...
    @overload
    @staticmethod
    def float(
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Sequence[Chromosome[builtins.float]],
    ) -> "Engine[builtins.float, list[np.ndarray]]": ...
    @staticmethod
    def float(
        shape: AtLeastOne[builtins.int] | None = None,
        *,
        init_range: tuple[builtins.float, builtins.float] | None = (0, 1.0),
        bounds: tuple[builtins.float, builtins.float] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = False,
        genes: AtLeastOne[Gene[builtins.float]] | None = None,
        chromosomes: AtLeastOne[Chromosome[builtins.float]] | None = None,
    ) -> "Engine[builtins.float, Any]": ...

    # ----------------------------
    # Int engine constructors
    # ----------------------------

    # Scalar
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, builtins.int]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = ...,
        genes: Gene[builtins.int],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, builtins.int]": ...

    # Vector via shape
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[builtins.int]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int8]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int16]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int32]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint8]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint16]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint32]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt64],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint64]]": ...
    @overload
    @staticmethod
    def int(
        shape: builtins.int,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int64]]": ...

    # Vector via genes
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[builtins.int]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int8],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int8]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int16],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int16]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int32],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int32]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt8],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint8]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt16],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint16]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt32],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint32]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt64],
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.uint64]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: Sequence[Gene[builtins.int]],
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, np.typing.NDArray[np.int64]]": ...

    # Vector via chromosome (dtype narrowing deferred)
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Chromosome[builtins.int],
    ) -> "Engine[builtins.int, list[builtins.int]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Chromosome[builtins.int],
    ) -> "Engine[builtins.int, np.ndarray]": ...

    # Matrix via shape
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[list[builtins.int]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.int8]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.int16]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[Int32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.int32]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt8],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.uint8]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt16],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.uint16]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt32],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.uint32]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: type[UInt64],
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.uint64]]]": ...
    @overload
    @staticmethod
    def int(
        shape: Sequence[builtins.int],
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: None = ...,
    ) -> "Engine[builtins.int, list[np.typing.NDArray[np.int64]]]": ...

    # Matrix via chromosomes (dtype narrowing deferred)
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[False] = False,
        genes: None = ...,
        chromosomes: Sequence[Chromosome[builtins.int]],
    ) -> "Engine[builtins.int, list[list[builtins.int]]]": ...
    @overload
    @staticmethod
    def int(
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: Literal[True],
        genes: None = ...,
        chromosomes: Sequence[Chromosome[builtins.int]],
    ) -> "Engine[builtins.int, list[np.ndarray]]": ...
    @staticmethod
    def int(
        shape: AtLeastOne[builtins.int] | None = None,
        *,
        init_range: tuple[builtins.int, builtins.int] | None = (0, 100),
        bounds: tuple[builtins.int, builtins.int] | None = None,
        dtype: RdDataType = ...,
        use_numpy: bool = False,
        genes: AtLeastOne[Gene[builtins.int]] | None = None,
        chromosomes: AtLeastOne[Chromosome[builtins.int]] | None = None,
    ) -> "Engine[builtins.int, Any]": ...

    # ----------------------------
    # Char engine constructors
    # ----------------------------

    @overload
    @staticmethod
    def char(
        shape: builtins.int,
        *,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, list[str]]": ...
    @overload
    @staticmethod
    def char(
        shape: Sequence[builtins.int],
        *,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, list[list[str]]]": ...
    @staticmethod
    def char(
        shape: AtLeastOne[builtins.int] = 1,
        char_set: str | list[str] | set[str] | None = None,
    ) -> "Engine[str, Any]": ...

    # ----------------------------
    # Bit engine constructors
    # ----------------------------

    @overload
    @staticmethod
    def bit(
        shape: builtins.int,
        use_numpy: Literal[False] = False,
    ) -> "Engine[bool, list[bool]]": ...
    @overload
    @staticmethod
    def bit(
        shape: builtins.int,
        use_numpy: Literal[True] = True,
    ) -> "Engine[bool, np.ndarray]": ...
    @overload
    @staticmethod
    def bit(
        shape: Sequence[builtins.int],
        use_numpy: Literal[False] = False,
    ) -> "Engine[bool, list[list[bool]]]": ...
    @overload
    @staticmethod
    def bit(
        shape: Sequence[builtins.int],
        use_numpy: Literal[True] = True,
    ) -> "Engine[bool, list[np.ndarray]]": ...
    @staticmethod
    def bit(
        shape: AtLeastOne[builtins.int] = 1,
        use_numpy: bool = False,
    ) -> "Engine[bool, Any]": ...
    @overload
    @staticmethod
    def packed_bit(
        num_bits: builtins.int,
        use_numpy: Literal[False] = False,
        *,
        words: Literal[False] = False,
    ) -> "Engine[bool, list[bool]]": ...
    @overload
    @staticmethod
    def packed_bit(
        num_bits: builtins.int,
        use_numpy: Literal[True],
        *,
        words: Literal[False] = False,
    ) -> "Engine[bool, np.ndarray]": ...
    @overload
    @staticmethod
    def packed_bit(
        num_bits: builtins.int,
        use_numpy: Literal[False] = False,
        *,
        words: Literal[True],
    ) -> "Engine[bool, list[builtins.int]]": ...
    @overload
    @staticmethod
    def packed_bit(
        num_bits: builtins.int,
        use_numpy: Literal[True],
        *,
        words: Literal[True],
    ) -> "Engine[bool, np.ndarray]": ...
    @staticmethod
    def packed_bit(
        num_bits: builtins.int,
        use_numpy: bool = False,
        *,
        words: bool = False,
    ) -> "Engine[bool, Any]": ...

    # ----------------------------
    # Other constructors
    # ----------------------------

    @staticmethod
    def permutation[P](items: list[P]) -> "Engine[P, list[P]]": ...
    @staticmethod
    def graph(
        shape: tuple[builtins.int, builtins.int],
        vertex: Op | list[Op] | None = None,
        edge: Op | list[Op] | None = None,
        output: Op | list[Op] | None = None,
        max_nodes: builtins.int | None = None,
        graph_type: str = "directed",
        dtype: RdDataType = Float64,
    ) -> "Engine[Op, Graph]": ...
    @staticmethod
    def tree(
        shape: tuple[builtins.int, builtins.int] = (1, 1),
        min_depth: builtins.int = 3,
        max_size: builtins.int = 30,
        vertex: Op | list[Op] | None = None,
        leaf: Op | list[Op] | None = None,
        root: Op | list[Op] | None = None,
        dtype: RdDataType = Float64,
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
        | tuple[builtins.int, str | Path, FileType | None]
        | CheckpointParam
        | None = None,
    ) -> Generation[G, T]: ...
    def nth(self, n: builtins.int) -> Generation[G, T]: ...

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
        frac: builtins.float | None = None,
    ) -> Self: ...
    def alter(self, *alters: AlterBase) -> Self: ...
    def diversity(
        self,
        diversity: Dist,
        threshold: Expr | builtins.float = 0.5,
        target: builtins.int | None = None,
    ) -> Self: ...
    def limit(self, *limits: Limit | Expr) -> Self: ...
    def filter(self, *filters: Filter) -> Self: ...
    def size(self, size: builtins.int) -> Self: ...
    def age(
        self,
        max_phenotype_age: builtins.int | None = None,
        max_species_age: builtins.int | None = None,
    ) -> Self: ...
    def population(self, population: Population[G]) -> Self: ...
    def minimizing(self) -> Self: ...
    def maximizing(self) -> Self: ...
    def objective(
        self,
        *obj: str,
        front_range: tuple[builtins.int, builtins.int] | None = None,
    ) -> Self: ...
    def front_range(self, min: builtins.int, max: builtins.int) -> Self: ...
    def parallel(self, num_workers: builtins.int | None = None) -> Self: ...
    def subscribe(self, *event_handler: Subscriber) -> Self: ...
    def generation(self, generation: Generation[G, T] | None) -> Self: ...
    def load_checkpoint(
        self, path: str | Path, ignore_not_found: bool = False
    ) -> Self: ...
    def write_checkpoint(
        self, path: str | Path, interval: builtins.int, file_type: FileType = "pkl"
    ) -> Self: ...
    def metrics(
        self, named_metrics: dict[str, Expr] | None = None, **kwargs
    ) -> Self: ...
