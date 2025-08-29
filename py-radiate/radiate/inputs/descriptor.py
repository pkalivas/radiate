from radiate.genome import GeneType
from .component import ComponentBase
from typing import Callable, Dict, Any, List


class DescriptorBase(ComponentBase):
    """
    Base class for all descriptors.
    """

    def __init__(
        self,
        component: str,
        args: Dict[str, Any] = {},
        allowed_genes: set[GeneType] | GeneType = {},
    ):
        super().__init__(component=component, args=args)
        if isinstance(allowed_genes, str):
            allowed_genes = {allowed_genes}
        self.allowed_genes = allowed_genes if allowed_genes else GeneType.all()

    def __repr__(self):
        return f"{self.__class__.__name__}(component={self.component}, input_type={self.input_type}, allowed_genes={self.allowed_genes})"


class CustomDescriptor(DescriptorBase):
    """
    Descriptor for custom data.
    """

    def __init__(self, descriptor: Callable[[Any], float | List[float]]):
        super().__init__(component="CustomDescriptor")
        self.descriptor = descriptor
