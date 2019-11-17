
## Radiate

Radiate is a parallel genetic programming engine capable of evolving solutions for supervised, unsupervised, and general reinforcement learning problems. 

Radiate follows the evolutionary algorithm found in [NEAT](http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) (Neuroevolution of Augmented Topologies), but seperates the the evolutionary process from the graph datastructure allowing users to evolve any defined datastructure. NEAT describes an evolutionary algorithm which follows speciation through fitness sharing thus allowing species to optimize within their own niche. 

Radiate exposes three traits to the user which (dpending on the problem) must be implemented. 
1. Genome * Genome wraps the structure to be evolved and makes the user implement two nessesary functions and one optional. Distance and crossover must be implemented but base is optional (depending on how the user chooses to fill the population).
2. Environment
  * Environment represnts the evolutionary enviromnet for the genome, this means it can contain simple statistics for the population's evoltuion, or parameters for crossing over and distance. Internally it is wrapped in a mutable threadsafe pointer so it is not intended to be shared for each genome, rather one per population. Environment requires no implementations of it's one function, however depending on the use case envionment exposes a function called reset which is intended to 'reset' the envionment.
3. Problem
  * Problem is what gives a genome it's fitness score. It requires two implemented functions: empty and solve. Empty is required and should return a base problem (think new()). Solve takes a genome and returns that genome's fitness score, so this is where the analyzing of the current state of the genome occurs. Checkout the examples folder for a few examples of how this can be implemented.
