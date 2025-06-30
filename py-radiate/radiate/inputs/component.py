from typing import Dict

class ComponentBase:

    def __init__(self, component: str, args: Dict[str, str] = {}):
        self.component = component
        self.args = args
