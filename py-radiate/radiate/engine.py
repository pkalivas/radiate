from radiate.radiate import FloatEngine
from radiate.codec import Codec
from radiate.selector import Selector

class Engine:

    def __init__(self,
                 codec=None | Codec,
                 Problem=None,
                 survivor_selector=None | Selector,
                 offspring_selector=None | Selector,
                 fitness_func = None,
                   **kwargs):
        self.engine = FloatEngine()
        pass
        

    def run(self, num_generations=1000):
        """Run the engine for a number of generations."""
        self.engine.run(num_generations)
        