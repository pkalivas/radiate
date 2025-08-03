from __future__ import annotations

from typing import Any, ClassVar
from abc import ABC

class PythonWrapper[T](ABC):
    """
    Abstract base class for Python wrapper objects that wrap Rust objects.
    Provides common functionality for conversion between Python and Rust objects.
    """
    
    # The name of the inner attribute that holds the Rust object
    __inner_attr__: ClassVar[str] = "_pyobj"
    
    # The type of the Rust object this wrapper wraps
    __rust_type__: ClassVar[type] = None
    
    def __init__(self):
        """
        Initialize the wrapper. Subclasses should override this method.
        """
        # Initialize the inner attribute if it doesn't exist
        if not hasattr(self, self.__inner_attr__):
            setattr(self, self.__inner_attr__, None)
    
    @classmethod
    def from_python(cls, py_obj: Any) -> 'PythonWrapper[T]':
        """
        Create an instance of the class from a Python/Rust object.
        
        Args:
            py_obj: The Python/Rust object to wrap
            
        Returns:
            An instance of the wrapper class
            
        Raises:
            TypeError: If py_obj is not of the expected type
        """
        if cls.__rust_type__ and not isinstance(py_obj, cls.__rust_type__):
            raise TypeError(f"Expected {cls.__rust_type__}, got {type(py_obj)}")
        
        instance = cls.__new__(cls)
        setattr(instance, cls.__inner_attr__, py_obj)
        return instance
    
    def to_python(self) -> Any:
        """
        Convert the wrapper back to the Python/Rust object.
        
        Returns:
            The underlying Python/Rust object
        """
        inner_attr = getattr(self, '__inner_attr__', '_pyobj')
        if not hasattr(self, inner_attr):
            raise AttributeError(f"{self.__class__.__name__} has no attribute '{inner_attr}'")
        
        return getattr(self, inner_attr)
    
    def __repr__(self) -> str:
        """Default representation using the inner object."""
        inner_attr = getattr(self, '__inner_attr__', '_pyobj')
        inner = getattr(self, inner_attr, None)
        return f"{self.__class__.__name__}({inner})"
    
    def __eq__(self, other: Any) -> bool:
        """Compare with another wrapper or the inner object."""
        if isinstance(other, type(self)):
            return self.to_python() == other.to_python()
        return self.to_python() == other
    
    def __hash__(self) -> int:
        """Hash based on the inner object."""
        return hash(self.to_python())

