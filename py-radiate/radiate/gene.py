# from dataclasses import dataclass
# from typing import Optional, TypeVar, Generic
# from abc import ABC, abstractmethod
# from radiate.radiate import FloatGene, IntGene, BitGene, CharGene, AnyGene

# T = TypeVar('T')

# @dataclass
# class GeneConfig:
#     type: str
#     num_genes: int = 1
#     num_chromosomes: int = 1
#     min_value: Optional[float | int] = None
#     max_value: Optional[float | int] = None
#     min_bound: Optional[float | int] = None
#     max_bound: Optional[float | int] = None
#     alleles: Optional[list] = None

# @dataclass
# class Gene:
#     """Base class for all gene types"""
    
#     def __init__(self,
#                     type: str = None,
#                     min_value: Optional[float | int] = None,
#                     max_value: Optional[float | int] = None,
#                     min_bound: Optional[float | int] = None,
#                     max_bound: Optional[float | int] = None,
#                     allele = None,
#                     **kwargs):
#         """Create a new gene with the given type and range."""
#         if type is not None:
#             gene_type = type.lower().strip()
#             if gene_type == 'float':
#                 self.__gene = PyGene.new_float(
#                     range=(float(min_value or 0.0), float(max_value or 1.0)),
#                     bounds=(float(min_bound or min_value or 0.0), float(max_bound or max_value or 1.0))
#                 )
#             elif gene_type == 'int':

#                 self.__gene = PyGene.new_int(
#                     range=(int(min_value or 0), int(max_value or 100)),
#                     bounds=(int(min_bound or min_value or 0), int(max_bound or max_value or 100))
#                 )
#             elif gene_type == 'bit':
#                 self.__gene = PyGene.new_bit()
#             elif gene_type == 'char':
#                 self.__gene = PyGene.new_char(allele=allele)
#             elif gene_type == 'any':
#                 self.__gene = PyGene.new_any(allele=allele)
#             else:
#                 raise ValueError(f"Unknown gene type: {type}")
#             return
        
#         # Type inference mode
#         if allele is not None:
#             # Infer from allele
#             if isinstance(allele, str) and len(allele) == 1:
#                 self.__gene = PyGene.new_char(allele=allele)
#             else:
#                 self.__gene = PyGene.new_any(allele=allele)
#             return

#         if min_value is not None and max_value is not None:
#             if isinstance(min_value, int) and isinstance(max_value, int):
#                 self.__gene = PyGene.new_int(
#                     range=(min_value, max_value),
#                     bounds=(min_bound or min_value, max_bound or max_value)
#                 )
#             elif isinstance(min_value, float) and isinstance(max_value, float):
#                 self.__gene = PyGene.new_float(
#                     range=(min_value, max_value),
#                     bounds=(min_bound or min_value, max_bound or max_value)
#                 )
#             else:
#                 raise TypeError("min_value and max_value must be of the same type (int or float)")
#         else:
#             raise ValueError("Unable to determine gene type — provide `type`, `allele`, or `min_value` and `max_value`")
    
#     def __repr__(self):
#         return f"Gene({self.__gene})"
    
#     def allele(self) -> T:
#         return self.__gene.allele()
    


    
# class GeneBase(ABC, Generic[T]):
#     """Base class for all gene types"""
    
#     @abstractmethod
#     def allele(self, index: int) -> T:
#         """Return the allele at the given index."""
#         pass

#     @abstractmethod
#     def with_allele(self, value: T):
#         """Return a new gene with the allele at the given index set to the given value."""
#         pass

#     @abstractmethod
#     def new_instance(self) -> 'GeneBase':
#         """Return a new instance of the gene."""
#         pass




