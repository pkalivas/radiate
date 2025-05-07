from typing import Dict, Any
from .param import EngineParam

class Selector(EngineParam):

    def __init__(self, name: str, args: Dict[str, Any] = None):
        """
        Initialize the selector with name and additional parameters.
        :param name: Name of the selector.
        :param args: Additional parameters for the selector.
        """
        super().__init__(name=name, args=args)


class TournamentSelector(Selector):

    def __init__(self, k: int = 3):
        """
        Initialize the tournament selector with tournament size.
        :param k: Tournament size.
        """
        super().__init__(
            name='tournament',
            args={'k': str(k)}
        )

class RouletteSelector(Selector):

    def __init__(self):
        """
        Initialize the roulette selector.
        """
        super().__init__(
            name='roulette'
        )

class RankSelector(Selector):

    def __init__(self):
        """
        Initialize the rank selector.
        """
        super().__init__(
            name='rank'
        )

class ElitismSelector(Selector):
    
    def __init__(self):
        """
        Initialize the elitism selector.
        """
        super().__init__(
            name='elitism'
        )




# class Selector:

#     def __init__(self, selector: PySelector):
#         self.selector = selector   
    
#     @staticmethod
#     def tournament(k=3):
#         """Create a tournament selector."""
#         selector = PySelector(
#             'tournament',
#             args={'k': str(k)}
#         )
#         return Selector(
#             selector
#         )
    
#     @staticmethod
#     def roulette():
#         """Create a roulette selector."""
#         selector = PySelector(
#             'roulette'
#         )
#         return Selector(
#             selector
#         )
    
#     @staticmethod
#     def rank():
#         """Create a rank selector."""
#         selector = PySelector(
#             'rank'
#         )
#         return Selector(
#             selector
#         )
    
#     @staticmethod
#     def elitism():
#         """Create an elitism selector."""
#         selector = PySelector(
#             'elitism'
#         )
#         return Selector(
#             selector
#         )
    
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