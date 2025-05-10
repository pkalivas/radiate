from typing import Dict, Any
from radiate.radiate import PyEngineParam


class EngineParam:
    """
    Class to hold engine parameters.
    """

    def __init__(self, name: str, args: Dict[str, Any] = None):
        args = args or {}
        self.params = PyEngineParam(
            name=name,
            args={k: str(v) for k, v in args.items()},
        )

    def __getattr__(self, name):
        """
        Get the value of an attribute.
        :param name: Name of the attribute.
        :return: Value of the attribute.
        """
        self.params.get_arg(name)

    def __repr__(self):
        args = ", ".join(f"{k}={v}" for k, v in self.params.get_args().items())
        return f"{self.name}({args})"

    def __str__(self):
        args = ", ".join(f"{k}={v}" for k, v in self.params.get_args().items())
        return f"{self.name}({args})"

    def name(self):
        return self.params.get_name()
