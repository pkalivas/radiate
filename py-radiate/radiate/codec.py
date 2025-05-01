from abc import ABC
from typing import Any, Dict, Optional, Union

class CodecBase(ABC):

    def __init__(self, gene_type: str, **kwargs):
        self._gene_type = gene_type
        self._kwargs = kwargs

    def encode(self) -> Any:
        """Encode the gene."""
        raise NotImplementedError("Subclasses must implement this method.")
    
    def decode(self) -> Any:
        """Decode the gene."""
        raise NotImplementedError("Subclasses must implement this method.")

class Codec:

    def __init__(self, type: str, **kwargs):
        self._type = type
        self._kwargs = kwargs


    def __repr__(self):
        return f"Codec({self._kwargs})"
    
    def gene_type(self):
        """Return the type of the gene."""
        return self._kwargs.get('type', None)
    
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
        """Create a codec for float genes."""
        if min_bound is None:
            min_bound = min_value
        if max_bound is None:
            max_bound = max_value
        return Codec(
            'float',
            num_genes=num_genes,
            num_chromosomes=num_chromosomes,
            min_value=min_value,
            max_value=max_value,
            min_bound=min_bound,
            max_bound=max_bound
        )
    