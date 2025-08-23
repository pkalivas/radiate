from .gene import (
    Gene,
    GeneType,
    FloatGene,
    IntGene,
    BitGene,
    CharGene,
)
from .chromosome import Chromosome
from .genotype import Genotype
from .phenotype import Phenotype
from .population import Population
from .species import Species
from .ecosystem import Ecosystem

from radiate.radiate import PyGeneType as gt

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
    "Genotype",
    "Chromosome",
    "Gene",
    "Phenotype",
    "Population",
    "Species",
    "Ecosystem",
    "FloatGene",
    "IntGene",
    "BitGene",
    "CharGene",
    "GENE_TYPE_MAPPING",
]
