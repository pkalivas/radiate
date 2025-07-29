from typing import Any, Dict

from .component import ComponentBase
from ..genome.gene import GeneType

class DescriptorBase(ComponentBase):
    """A class representing a descriptor for a problem."""

    def __init__(
        self,
        component: str,
        args: Dict[str, Any] = {},
        allowed_genes: set[str] | str = {},
    ):
        """
        Initialize the diversity parameter with a PyDiversity instance.
        :param diversity: An instance of PyDiversity.
        """
        super().__init__(component=component, args=args)
        if isinstance(allowed_genes, str):
            allowed_genes = {allowed_genes}
        self.allowed_genes = allowed_genes if allowed_genes else GeneType.ALL


class FitnessDescriptor(DescriptorBase):
    """A class representing a fitness descriptor for novelty search."""

    def __init__(self, component: str, allowed_genes: set[str] | str = {}, **kwargs):
        """
        Initialize the fitness descriptor.
        """
        super().__init__(component=component, allowed_genes=allowed_genes, **kwargs)
