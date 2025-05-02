from dataclasses import dataclass, field
from typing import Any, Optional, Dict

from radiate._typing import GeneType


@dataclass
class Genome:
    """Configuration for the genome"""
    
    # Gene Configuration
    gene_type: GeneType
    num_genes: int = 1
    num_chromosomes: int = 1
    min_value: Optional[float | int] = None
    max_value: Optional[float | int] = None
    min_bound: Optional[float | int] = None
    max_bound: Optional[float | int] = None
    alleles: Optional[Any] = None
    
    # Additional Parameters
    extra_params: Dict[str, Any] = field(default_factory=dict)


    def __getattr__(self, name):
        """Get the value of an attribute."""
        # This is only called for attributes that don't exist
        if name in self.extra_params:
            return self.extra_params[name]
        raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")
    

    def __setattr__(self, name, value):
        """Set the value of an attribute."""
        # For dataclass fields, use normal attribute setting
        if name in self.__annotations__ or name in self.__dict__:
            super().__setattr__(name, value)
        else:
            # For extra parameters, store in extra_params
            self.extra_params[name] = value

           