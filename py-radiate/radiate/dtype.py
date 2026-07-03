from __future__ import annotations

from inspect import isclass
from typing import Iterator

from radiate.radiate import _get_dtype_max, _get_dtype_min


class Field:
    """
    Definition of a single field within a `Struct` DataType.

    Parameters
    ----------
    name
        The name of the field within its parent `Struct`.
    dtype
        The `DataType` of the field's values.
    """

    name: str
    dtype: DataType | DataTypeClass

    def __init__(self, name: str, dtype: DataType | DataTypeClass) -> None:
        self.name = name
        self.dtype = dtype

    def __eq__(self, other) -> bool:  # type: ignore[override]
        return (self.name == other.name) & (self.dtype == other.dtype)

    def __hash__(self) -> int:
        return hash((self.name, self.dtype))

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.name!r}, {self.dtype})"


class DataTypeClass(type):
    def __getitem__(cls, item):
        return cls(item)

    def __str__(cls):
        return cls.__name__


class DataType(metaclass=DataTypeClass):
    def __init__(self):
        self.name = self.__class__.__name__

    def __repr__(self) -> str:
        return self.name

    def __str__(self) -> str:
        return self.name

    def __eq__(self, value):
        if issubclass(type(value), DataType):
            return self.name == value.name
        if type(value) is DataTypeClass:
            return issubclass(value, type(self))
        else:
            return isinstance(value, type(self))


class NumericType(DataType):
    @classmethod
    def max(cls) -> int | float:
        return _get_dtype_max(cls.__name__)

    @classmethod
    def min(cls) -> int | float:
        return _get_dtype_min(cls.__name__)

    @classmethod
    def is_int(cls) -> bool:
        return issubclass(cls, IntegerType)

    @classmethod
    def is_float(cls) -> bool:
        return issubclass(cls, FloatType)


class IntegerType(NumericType):
    """Integer data type."""


class FloatType(NumericType):
    """Floating-point data type."""


class UInt8(IntegerType):
    """Unsigned 8-bit integer."""


class UInt16(IntegerType):
    """Unsigned 16-bit integer."""


class UInt32(IntegerType):
    """Unsigned 32-bit integer."""


class UInt64(IntegerType):
    """Unsigned 64-bit integer."""


class UInt128(IntegerType):
    """Unsigned 128-bit integer."""


class Int8(IntegerType):
    """Signed 8-bit integer."""


class Int16(IntegerType):
    """Signed 16-bit integer."""


class Int32(IntegerType):
    """Signed 32-bit integer."""


class Int64(IntegerType):
    """Signed 64-bit integer."""


class Int128(IntegerType):
    """Signed 128-bit integer."""


class Float32(FloatType):
    """32-bit floating-point number."""


class Float64(FloatType):
    """64-bit floating-point number."""


class Boolean(DataType):
    """Boolean data type."""


class String(DataType):
    """UTF-8 encoded string type."""


class Char(DataType):
    """Single character string type."""


class Null(DataType):
    """Null type, representing the absence of a value."""


class NestedType(DataType):
    """Base class for nested data types."""


class Struct(NestedType):
    """
        Struct data type, representing a collection of named fields.
        Parameters
    ----------
    fields    A list of `Field` instances defining the structure of the `Struct`.
    Examples
    --------
    >>> person_dtype = Struct(
    ...     "Person",
    ...     [
    ...         Field("name", String),
    ...         Field("age", Int32),
    ...         Field("is_student", Bool),
    ...     ],
    ... )
    >>> person_dtype
    Struct(Person, {'name': String, 'age': Int32, 'is_student': Bool})
    """

    name: str
    fields: list[Field]

    def __init__(
        self,
        name: str,
        fields: list[Field] | list[tuple[str, DataType | DataTypeClass]],
    ) -> None:
        self.name = name
        if all(isinstance(fld, Field) for fld in fields):
            self.fields = fields  # type: ignore[assignment]
        elif all(isinstance(fld, tuple) and len(fld) == 2 for fld in fields):
            self.fields = [Field(name, dtype) for name, dtype in fields]  # type: ignore[assignment]
        else:
            raise ValueError(
                "Fields must be a list of Field instances or a list of (name, dtype) tuples."
            )

    def __eq__(self, other) -> bool:  # type: ignore[override]
        if isclass(other) and issubclass(other, Struct):
            return True
        elif isinstance(other, Struct):
            return self.fields == other.fields and self.name == other.name
        else:
            return False

    def __hash__(self) -> int:
        return hash((self.__class__, tuple(self.fields), self.name))

    def __iter__(self) -> Iterator[tuple[str, DataType | DataTypeClass]]:
        for fld in self.fields:
            yield fld.name, fld.dtype

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.name}, {dict(self)})"

    def __str__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.name}, {dict(self)})"


class List(NestedType):
    """
        List data type, representing a homogeneous collection of values.
        Parameters
    ----------
    inner    The `DataType` of the values contained within the list. If not specified, the list is considered to be of
      an unspecified inner type, and will compare as equal to any other List type (eg: List[Int32] == List == List[Float64]).
    Examples
    --------
    >>> int_list_dtype = List(Int32)
    >>> int_list_dtype
    List(Int32)
    >>> int_list_dtype == List(Int32)
    True
    >>> int_list_dtype == List(Float64)
    False
    >>> int_list_dtype == List
    True
    >>> List == List(Int32)
    True
    """

    inner: DataTypeClass | DataType

    def __init__(self, inner: DataTypeClass | DataType) -> None:
        self.inner = inner

    def __eq__(self, other: DataTypeClass | DataType) -> bool:  # type: ignore[override]
        if type(other) is DataTypeClass and issubclass(other, List):
            return True
        elif isinstance(other, List):
            return self.inner == other.inner
        else:
            return False

    def __hash__(self) -> int:
        return hash((self.__class__, self.inner))

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.inner!r})"

    def __str__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({self.inner})"


class Dict(NestedType):
    """
    Dict data type, representing a runtime container of named fields.

    Unlike `Struct`, a `Dict` is not a declared schema — it describes the
    shape of a Python dict instance, where each key may have its own value
    type but the shape belongs to the instance, not the type.

    Parameters
    ----------
    fields    A list of `Field` instances (or `(name, dtype)` tuples) describing
              the key→type pairs present in the dict.

    Examples
    --------
    >>> d = Dict([Field("loss", Float32), Field("epoch", Int32)])
    >>> d
    Dict({'loss': Float32, 'epoch': Int32})
    """

    fields: list[Field]

    def __init__(
        self,
        fields: list[Field] | list[tuple[str, DataType | DataTypeClass]],
    ) -> None:
        if all(isinstance(fld, Field) for fld in fields):
            self.fields = fields  # type: ignore[assignment]
        elif all(isinstance(fld, tuple) and len(fld) == 2 for fld in fields):
            self.fields = [Field(name, dtype) for name, dtype in fields]  # type: ignore[assignment]
        else:
            raise ValueError(
                "Fields must be a list of Field instances or a list of (name, dtype) tuples."
            )

    def __eq__(self, other) -> bool:  # type: ignore[override]
        if isclass(other) and issubclass(other, Dict):
            return True
        elif isinstance(other, Dict):
            return self.fields == other.fields
        else:
            return False

    def __hash__(self) -> int:
        return hash((self.__class__, tuple(self.fields)))

    def __iter__(self) -> Iterator[tuple[str, DataType | DataTypeClass]]:
        for fld in self.fields:
            yield fld.name, fld.dtype

    def __repr__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({dict(self)})"

    def __str__(self) -> str:
        class_name = self.__class__.__name__
        return f"{class_name}({dict(self)})"
