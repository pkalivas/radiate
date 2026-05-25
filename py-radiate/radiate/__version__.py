from radiate.radiate import py_version

__version__ = py_version()
__version_tuple__ = tuple(map(int, __version__.split(".")))
