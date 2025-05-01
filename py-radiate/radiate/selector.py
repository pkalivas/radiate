
class Selector:

    def __init__(self, type: str, **kwargs):
        self._type = type
        self._kwargs = kwargs

    def __repr__(self):
        return f"Selector({self._kwargs})"
    
    def __getattr__(self, name):
        """Get the value of an attribute."""
        if name in self._kwargs:
            return self._kwargs[name]
        raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")
    
    @staticmethod
    def tournament(k=3):
        """Create a tournament selector."""
        return Selector(
            'tournament',
            k=k
        )
    
    @staticmethod
    def roulette():
        """Create a roulette selector."""
        return Selector(
            'roulette'
        )
    
    @staticmethod
    def rank():
        """Create a rank selector."""
        return Selector(
            'rank'
        )
    
    @staticmethod
    def elitism():
        """Create an elitism selector."""
        return Selector(
            'elitism'
        )
    
    @staticmethod
    def boltzmann(temp=1.0):
        """Create a Boltzmann selector."""
        return Selector(
            'boltzmann',
            temp=temp
        )