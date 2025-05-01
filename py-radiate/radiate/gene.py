from dataclasses import dataclass
from typing import Optional, TypeVar, Generic
from abc import ABC, abstractmethod

T = TypeVar('T')

@dataclass
class GeneConfig:
    type: str
    num_genes: int = 1
    num_chromosomes: int = 1
    min_value: Optional[float | int] = None
    max_value: Optional[float | int] = None
    min_bound: Optional[float | int] = None
    max_bound: Optional[float | int] = None
    alleles: Optional[list] = None

@dataclass
class Gene:
    """Base class for all gene types"""
    
    def __init__(self, config: GeneConfig):
        self.config = config
        self._type = config.type
        self._kwargs = {k: v for k, v in config.__dict__.items() if v is not None}
    
    def __repr__(self):
        return f"Gene({self._kwargs})"
    
    def __getattr__(self, name):
        """Get the value of an attribute."""
        if name in self._kwargs:
            return self._kwargs[name]
        raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")
    
    @staticmethod
    def float(num_genes=1, 
              num_chromosomes=1,
              min_value=0.0,
              max_value=1.0, 
              min_bound=None, 
              max_bound=None):
        """Create a gene for float values."""
        if min_bound is None:
            min_bound = min_value
        if max_bound is None:
            max_bound = max_value
        return Gene(
            GeneConfig(
                type='float',
                num_genes=num_genes,
                num_chromosomes=num_chromosomes,
                min_value=min_value,
                max_value=max_value,
                min_bound=min_bound,
                max_bound=max_bound
            )
        )

    
class GeneBase(ABC, Generic[T]):
    """Base class for all gene types"""
    
    @abstractmethod
    def allele(self, index: int) -> T:
        """Return the allele at the given index."""
        pass

    @abstractmethod
    def with_allele(self, value: T):
        """Return a new gene with the allele at the given index set to the given value."""
        pass

    @abstractmethod
    def new_instance(self) -> 'GeneBase':
        """Return a new instance of the gene."""
        pass




