from radiate.radiate import PySelector

class Selector:

    def __init__(self, selector: PySelector):
        self.selector = selector   
    
    @staticmethod
    def tournament(k=3):
        """Create a tournament selector."""
        selector = PySelector(
            'tournament',
            args={'k': str(k)}
        )
        return Selector(
            selector
        )
    
    @staticmethod
    def roulette():
        """Create a roulette selector."""
        selector = PySelector(
            'roulette'
        )
        return Selector(
            selector
        )
    
    @staticmethod
    def rank():
        """Create a rank selector."""
        selector = PySelector(
            'rank'
        )
        return Selector(
            selector
        )
    
    @staticmethod
    def elitism():
        """Create an elitism selector."""
        selector = PySelector(
            'elitism'
        )
        return Selector(
            selector
        )
    
    # @staticmethod
    # def roulette():
    #     """Create a roulette selector."""
    #     return Selector(
    #         'roulette'
    #     )
    
    # @staticmethod
    # def rank():
    #     """Create a rank selector."""
    #     return Selector(
    #         'rank'
    #     )
    
    # @staticmethod
    # def elitism():
    #     """Create an elitism selector."""
    #     return Selector(
    #         'elitism'
    #     )
    
    # @staticmethod
    # def boltzmann(temp=1.0):
    #     """Create a Boltzmann selector."""
    #     return Selector(
    #         'boltzmann',
    #         temp=temp
    #     )