from __future__ import annotations

from typing import TYPE_CHECKING, Any, Callable

from radiate.genome.chromosome import Chromosome
from radiate.genome.gene import Gene
from radiate.gp.op import Op

from .handlers import EventHandler

if TYPE_CHECKING:
    from radiate.codec.float import FloatCodec
    from radiate.codec.int import IntCodec
    from radiate.codec.char import CharCodec
    from radiate.codec.bit import BitCodec


type Subscriber = (
    Callable[[Any], None]
    | list[Callable[[Any], None]]
    | EventHandler
    | list[EventHandler]
)

type NodeValues = list[Op] | Op | list[str] | str
