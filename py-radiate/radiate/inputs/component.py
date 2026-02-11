from typing import Any, Dict
from dataclasses import dataclass


class ComponentBase:
    def __init__(self, component: str, args: Dict[str, Any] = {}):
        self.component = component
        self.args = args
