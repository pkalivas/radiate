# import radiate._reexpr as re

# import inspect
# for name, obj in inspect.getmembers(re):
#     if inspect.isclass(obj):
#         print(name, obj)

from typing import TypeAlias, Any, Union

GeneType: TypeAlias = Union[type[float], type[int], type[bool], type[str], type[Any]]