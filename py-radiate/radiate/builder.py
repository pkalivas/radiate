from typing import Dict, Any, List, Union
from ._typing import GeneType
from .genome import Genome

class EngineBuilder:
    genome: Genome
    args: Dict[str, Any] = {}

    def __init__(self, genome: None | Genome = None, **kwargs):
        """Initialize the builder with genome."""
        if genome is None:
            raise ValueError("Genome must be provided.")
        self.gene_type = genome.gene_type
        self.args = kwargs


class FloatEngineBuilder(EngineBuilder):
    """
    Builder for float engine.
    """
    def __init__(self, genome: None | Genome = None, **kwargs):
        """Initialize the builder with genome."""
        super().__init__(genome, **kwargs)
        if self.gene_type is not GeneType.FLOAT:
            raise TypeError(f"Genome type {self.gene_type} is not supported.")
        