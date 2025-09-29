from __future__ import annotations

from .gene import (
    Gene,
    AnyGene,
)

from .chromosome import Chromosome
from .genotype import Genotype
from .phenotype import Phenotype
from .population import Population
from .species import Species
from .ecosystem import Ecosystem

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
    ANY = "AnyGene"

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
            GeneType.ANY,
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
            case "anygene":
                return GeneType.ANY
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
        gt.AnyGene: GeneType.ANY,
    },
    "rs": {
        GeneType.FLOAT: gt.Float,
        GeneType.INT: gt.Int,
        GeneType.BIT: gt.Bit,
        GeneType.CHAR: gt.Char,
        GeneType.GRAPH: gt.GraphNode,
        GeneType.TREE: gt.TreeNode,
        GeneType.PERMUTATION: gt.Permutation,
        GeneType.ANY: gt.AnyGene,
    },
}


__all__ = [
    "GeneType",
    "Genotype",
    "Chromosome",
    "Gene",
    "Phenotype",
    "Population",
    "Species",
    "AnyGene",
    "Ecosystem",
    "GENE_TYPE_MAPPING",
]
