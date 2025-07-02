# Examples Examples

## Maze Runner

Maze Solving with Permutation Codec

This example demonstrates using the PermutationCodec to solve a maze navigation problem.
We have a set of waypoints in a maze and need to find the optimal order to visit them.

**File:** `maze_runner.py`

Main function to run the maze solving example.

### Code Snippet

```python
def main():
    """Main function to run the maze solving example."""
    
    maze_solver = create_sample_maze()
    
    print("Maze Waypoints:")
    for i, wp in enumerate(maze_solver.waypoints):
        print(f"  {i}: {wp}")
    print()
    
    result = run_maze_evolution(maze_solver, generations=250)

    best_genotype = result.value()
    best_permutation = best_genotype 
    best_fitness = result.score()
    
    print("\nBest solution found:")
    print(f"  Fitness: {best_fitness}")
    print(f"  Path length: {best_fitness}")
    print(f"  Permutation: {best_permutation}")
```

---

## Nn

**File:** `nn.py`

### Code Snippet

```python
# #!/usr/bin/env python3
# """
# Neural Network Evolution Performance Test
# Tests the impact of NumPy on genetic algorithm performance
# """

# import os
# import sys
# import time
# import numpy as np
# from typing import List, Tuple



# project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../py-radiate"))
# sys.path.insert(0, project_root)

# print(f"Project root: {project_root}")

# import radiate as rd
```

---

