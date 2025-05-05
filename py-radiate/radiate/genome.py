from dataclasses import dataclass, field
from typing import Any, Optional, Dict

from radiate._typing import GeneType


class Genome:
    """Configuration for the genome"""
    gene_type: GeneType
    args: Dict[str, Any] = field(default_factory=dict)

    def __init__(self, gene_type: GeneType, **kwargs):
        """
        Initialize the genome with gene type and additional parameters.
        :param gene_type: Type of the genome.
        :param kwargs: Additional parameters for the genome.
        """
        self.gene_type = gene_type
        self.args = kwargs


class FloatGenome(Genome):
    """Configuration for a float genome"""

    DEFAULT_MIN_VALUE: float = 0.0
    DEFAULT_MAX_VALUE: float = 1.0
    
    gene_type: GeneType = GeneType.FLOAT
    num_genes: int = 1
    num_chromosomes: int = 1
    min_value: Optional[float] = None
    max_value: Optional[float] = None
    min_bound: Optional[float] = None
    max_bound: Optional[float] = None

    def __init__(self, 
                 num_genes: int = 1, 
                 num_chromosomes: int = 1, 
                 min_value: Optional[float] = None, 
                 max_value: Optional[float] = None, 
                 min_bound: Optional[float] = None, 
                 max_bound: Optional[float] = None):
        """
        Initialize the float genome with number of genes, chromosomes, and value bounds.
        :param num_genes: Number of genes in the genome.
        :param num_chromosomes: Number of chromosomes in the genome.
        :param min_value: Minimum value for the genes.
        :param max_value: Maximum value for the genes.
        :param min_bound: Minimum bound for the genes.
        :param max_bound: Maximum bound for the genes.
        """
        super().__init__(gene_type=GeneType.FLOAT)
        self.num_genes = num_genes
        self.num_chromosomes = num_chromosomes
        self.min_value = min_value
        self.max_value = max_value
        self.min_bound = min_bound
        self.max_bound = max_bound

        if self.min_value is None:
            self.min_value = self.DEFAULT_MIN_VALUE
        if self.max_value is None:
            self.max_value = self.DEFAULT_MAX_VALUE
        if self.min_bound is None:
            self.min_bound = self.min_value
        if self.max_bound is None:
            self.max_bound = self.max_value

        
            