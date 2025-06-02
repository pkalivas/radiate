import abc

from typing import Any, Dict, Optional, List

from ..genome import Chromosome


class CodecBase(abc.ABC):

    @abc.abstractmethod
    def encode(self) -> List[Any]:
        pass

    

