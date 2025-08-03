from __future__ import annotations

from typing import TYPE_CHECKING, Any, Callable, List, Union

from radiate.genome.chromosome import Chromosome
from radiate.genome.gene import Gene
from radiate.gp.op import Op

from .handlers import EventHandler

if TYPE_CHECKING:
    from radiate.codec.float import FloatCodec

type Subscriber = Union[
    Callable[[Any], None], List[Callable[[Any], None]], EventHandler, List[EventHandler]
]

type NodeValues = Union[List[Op], Op, List[str], str]

type FloatEncoding = Union[
    "FloatCodec",
    List[Gene[float]],
    Chromosome[float],
    List[Chromosome[float]],
]
