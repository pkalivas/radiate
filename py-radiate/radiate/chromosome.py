# from dataclasses import dataclass
# from typing import Optional, TypeVar, Generic
# from abc import ABC, abstractmethod
# from radiate.radiate import PyChromosome

# class Chromosome:
#     """
#     A class representing a chromosome in a genetic algorithm.
#     """

#     def __init__(self,
#                     num_genes: int = 1,
#                     type: str = None,
#                     min_value: Optional[float | int] = None,
#                     max_value: Optional[float | int] = None,
#                     min_bound: Optional[float | int] = None,
#                     max_bound: Optional[float | int] = None,
#                     alleles = None,
#                     **kwargs):
#         """
#         Initialize the chromosome with a list of genes.

#         :param genes: List of genes (optional)
#         """

#         if type is not None:
#             gene_type = type.lower().strip()
#             if gene_type == 'float':
#                 self.__chromosome = PyChromosome.new_float(
#                     num_genes=num_genes,
#                     range=(float(min_value or 0.0), float(max_value or 1.0)),
#                     bounds=(float(min_bound or min_value or 0.0), float(max_bound or max_value or 1.0))
#                 )
#             elif gene_type == 'int':
#                 self.__chromosome = PyChromosome.new_int(
#                     num_genes=num_genes,
#                     range=(int(min_value or 0), int(max_value or 100)),
#                     bounds=(int(min_bound or min_value or 0), int(max_bound or max_value or 100))
#                 )
#             elif gene_type == 'bit':
#                 self.__chromosome = PyChromosome.new_bit(num_genes=num_genes)
#             elif gene_type == 'char':
#                 self.__chromosome = PyChromosome.new_char(num_genes=num_genes, allele=alleles)
#             elif gene_type == 'any':
#                 self.__chromosome = PyChromosome.new_any(num_genes=num_genes, allele=alleles)
#             return
        
#         if alleles is not None:
#             if isinstance(alleles, str) and len(alleles) == 1:
#                 self.__chromosome = PyChromosome.new_char(num_genes=num_genes, allele=alleles)
#             else:
#                 self.__chromosome = PyChromosome.new_any(num_genes=num_genes, allele=alleles)
#         else:
#             if min_value is not None and max_value is not None:
#                 self.__chromosome = PyChromosome.new_float(
#                     num_genes=num_genes,
#                     range=(float(min_value), float(max_value)),
#                     bounds=(float(min_bound or min_value), float(max_bound or max_value))
#                 )
#             else:
#                 self.__chromosome = PyChromosome.new_int(
#                     num_genes=num_genes,
#                     range=(int(min_value or 0), int(max_value or 100)),
#                     bounds=(int(min_bound or min_value or 0), int(max_bound or max_value or 100))
#                 )

#     def __iter__(self):
#         """
#         Iterate over the genes in the chromosome.

#         :return: An iterator over the genes
#         """
#         count = 0
#         while count < len(self.__chromosome):
#             yield self.__chromosome[count]
#             count += 1

#     def __getitem__(self, index):
#         """
#         Get the gene at the specified index.

#         :param index: Index of the gene
#         :return: Gene at the specified index
#         """
#         return self.__chromosome[index]
    
#     def __len__(self):
#         """
#         Get the number of genes in the chromosome.

#         :return: Number of genes
#         """
#         return len(self.__chromosome)