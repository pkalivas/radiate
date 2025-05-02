# problem.py
from typing import Generic, Callable, Any

class Problem:

    def __init__(self, fitness_fn: Callable[[Any], Any] = None):
        self.fitness_fn = fitness_fn

