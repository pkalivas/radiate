from __future__ import annotations

from typing import TYPE_CHECKING, Any
from enum import Enum

from radiate.radiate import PyGeneType as gt


class GeneType(Enum):
    FLOAT = "FloatGene"
    INT = "IntGene"
    BIT = "BitGene"
    CHAR = "CharGene"
    PERMUTATION = "PermutationGene"
    GRAPH = "GraphNode"
    TREE = "TreeNode"

    @staticmethod
    def all() -> set[GeneType]:
        return {
            GeneType.FLOAT,
            GeneType.INT,
            GeneType.BIT,
            GeneType.CHAR,
            GeneType.PERMUTATION,
            GeneType.GRAPH,
            GeneType.TREE,
        }

    @staticmethod
    def core() -> set[GeneType]:
        return {
            GeneType.FLOAT,
            GeneType.INT,
            GeneType.BIT,
            GeneType.CHAR,
            GeneType.PERMUTATION,
        }

    @staticmethod
    def from_str(gene_type: str) -> GeneType:
        type_lower = str(gene_type).lower()
        match type_lower:
            case "floatgene":
                return GeneType.FLOAT
            case "intgene":
                return GeneType.INT
            case "bitgene":
                return GeneType.BIT
            case "chargene":
                return GeneType.CHAR
            case "permutationgene":
                return GeneType.PERMUTATION
            case "graphnode":
                return GeneType.GRAPH
            case "treenode":
                return GeneType.TREE
            case _:
                raise ValueError(f"Invalid gene type: {gene_type}")


GENE_TYPE_MAPPING = {
    "py": {
        gt.Float: GeneType.FLOAT,
        gt.Int: GeneType.INT,
        gt.Bit: GeneType.BIT,
        gt.Char: GeneType.CHAR,
        gt.GraphNode: GeneType.GRAPH,
        gt.TreeNode: GeneType.TREE,
        gt.Permutation: GeneType.PERMUTATION,
    },
    "rs": {
        GeneType.FLOAT: gt.Float,
        GeneType.INT: gt.Int,
        GeneType.BIT: gt.Bit,
        GeneType.CHAR: gt.Char,
        GeneType.GRAPH: gt.GraphNode,
        GeneType.TREE: gt.TreeNode,
        GeneType.PERMUTATION: gt.Permutation,
    },
}


__all__ = [
    "GeneType",
    "Gene",
    "Chromosome",
    "Genotype",
    "Phenotype",
    "Population",
    "Species",
    "Ecosystem",
]

if TYPE_CHECKING:
    # These are for IDE/type-checker only; no runtime import cycles.
    from .gene import Gene
    from .chromosome import Chromosome
    from .genotype import Genotype
    from .phenotype import Phenotype
    from .population import Population
    from .species import Species
    from .ecosystem import Ecosystem


_LAZY = {
    "Gene": (".gene", "Gene"),
    "Chromosome": (".chromosome", "Chromosome"),
    "Genotype": (".genotype", "Genotype"),
    "Phenotype": (".phenotype", "Phenotype"),
    "Population": (".population", "Population"),
    "Species": (".species", "Species"),
    "Ecosystem": (".ecosystem", "Ecosystem"),
}


def __getattr__(name: str) -> Any:
    if name in _LAZY:
        from importlib import import_module

        mod_name, sym = _LAZY[name]
        mod = import_module(mod_name, __name__)
        val = getattr(mod, sym)
        globals()[name] = val  # cache
        return val

    msg = f"module {__name__!r} has no attribute {name!r}"
    raise AttributeError(msg)
