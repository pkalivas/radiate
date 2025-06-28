
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
    
    @staticmethod
    def relu() -> "Op":
        return Op("relu")
    
    @staticmethod
    def tanh() -> "Op":
        return Op("tanh")
    
    @staticmethod
    def linear() -> "Op":
        return Op("linear")
    
    @staticmethod
    def sum() -> "Op":
        return Op("sum")
    
    @staticmethod
    def prod() -> "Op":
        return Op("prod")
    
    @staticmethod
    def diff() -> "Op":
        return Op("diff")
    
    @staticmethod
    def pow() -> "Op":
        return Op("pow")
    
    @staticmethod
    def log() -> "Op":
        return Op("log")
    
    @staticmethod
    def sin() -> "Op":
        return Op("sin")
    
    @staticmethod
    def cos() -> "Op":
        return Op("cos")
    
    @staticmethod
    def identity() -> "Op":
        return Op("identity")
    
    @staticmethod
    def all_ops() -> list["Op"]:
        return [
            Op.add(),
            Op.sub(),
            Op.mul(),
            Op.div(),
            Op.linear(),
            Op.relu(),
            Op.tanh(),
            Op.sigmoid(),
            Op.sum(),
            Op.prod(),
            Op.diff(),
            Op.pow(),
            Op.log(),
            Op.sin(),
            Op.cos(),
            Op.identity(),
            Op.weight(),
        ]
