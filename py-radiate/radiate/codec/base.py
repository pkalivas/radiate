from __future__ import annotations

from typing import TYPE_CHECKING
from abc import ABC, abstractmethod

from radiate._bridge.wrapper import RsObject


if TYPE_CHECKING:
    from radiate.genome import Genotype, GeneType


class CodecBase[T, D](RsObject, ABC):
    gene_type: "GeneType"

    @abstractmethod
    def encode(self) -> "Genotype[T]":
        raise NotImplementedError("encode method must be implemented by subclasses.")

    @abstractmethod
    def decode(self, genotype: "Genotype[T]") -> D:
        raise NotImplementedError("decode method must be implemented by subclasses.")
