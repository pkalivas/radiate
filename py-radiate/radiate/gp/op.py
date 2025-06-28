
class Op:

    def __init__(self, name: str, **kwargs):
        self.name = name
        self.args = kwargs

    def __repr__(self):
        if self.args:
            args_str = ", ".join(f"{k}={v}" for k, v in self.args.items())
            return f"Op({self.name}, {args_str})"
        return f"Op({self.name})"
    
    def __getitem__(self, key):
        return self.args.get(key, None)
    

    @staticmethod
    def var(idx: int = 0) -> "Op":
        return Op("var", index=idx)

    @staticmethod
    def const(value: float) -> "Op":
        return Op("const", value=value)
    
    @staticmethod
    def add() -> "Op":
        return Op("add")
    
    @staticmethod
    def sub() -> "Op":
        return Op("sub")
    
    @staticmethod
    def mul() -> "Op":
        return Op("mul")
    
    @staticmethod
    def div() -> "Op":
        return Op("div")
    
    @staticmethod
    def sigmoid() -> "Op":
        return Op("sigmoid")
    
    @staticmethod
    def weight() -> "Op":
        return Op("weight")
