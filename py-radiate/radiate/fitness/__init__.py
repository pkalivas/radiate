
from .base import FitnessBase
from .regression import Regression
from .custom import CallableFitness
from .novelty import NoveltySearch
from .compiler import fitness

# Only expose for testing
def __getattr__(name):
    """Lazy import for test-only modules."""
    if name == "FitnessCompiler":
        import os
        if os.environ.get("RADIATE_TESTING") or "pytest" in os.environ.get("_", ""):
            from .compiler import FitnessCompiler
            return FitnessCompiler
    elif name == "CompiledFitness":
        import os
        if os.environ.get("RADIATE_TESTING") or "pytest" in os.environ.get("_", ""):
            from .compiler import CompiledFitness
            return CompiledFitness
        raise AttributeError(f"module '{__name__}' has no attribute '{name}'")
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")

__all__ = [
    "FitnessBase",
    "Regression", 
    "CallableFitness",
    "NoveltySearch",
    "fitness",
]