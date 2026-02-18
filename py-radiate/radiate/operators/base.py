from typing import Any, Dict


class ComponentBase:
    def __init__(self, component: str, args: Dict[str, Any] = {}):
        self.component = component
        self.args = args
