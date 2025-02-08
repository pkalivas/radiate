
# Genetic Programming 

The `radiate-gp` crate provides the fundamental building blocks for [Genetic Programming](https://en.wikipedia.org/wiki/Genetic_programming) (GP). Mainly, it provides data structures and algorithms to build and evolve `Tree`s and `Graph`s. Traditionally, GP is a method of evolving 'programs' to solve problems. The idea is to represent a program as a `Tree` or a `Graph` and evolve it to solve a specific problem. These problems are typically some sort of regression problems, but can be used to solve any problem that can be represented as a `Tree` or a `Graph`. It can be easier to think of these as evolving Expression Trees/Random Forests/Decision Trees or Neural Networks. 

The crate is not yet documented to the extent it should be, but you can refer to the [examples](https://github.com/pkalivas/radiate/tree/master/examples) for now.

## Nodes

The `Node` is the `Gene` of both the `Graph` and the `Tree`. It `Allele` is an
enum, `Ops<T>`, that provides a number of different ways to represent the node's.

!!! note 

    This needs a lot more documentation.

## Graphs

Graphs are a powerful way to represent problems. They are used in many fields, such as neural networks, and can be used to solve complex problems. Radiate offers an extremely unique way to build any graph architecture you can think of though the
`Architect<'a, C, T>` and integrate it seemlessly with the `GeneticEngine`. 

The `Architect` is a builder pattern that allows you to layer, attach, and build any graph architecture you can think of. Currently it has pre-built functionality for building `Graph`s:

* Acyclic
* Weighted Acyclic
* Cyclic
* Weighted Cyclic
* Attention Units
* Hopfield Networks
* Long-Short Term Memory (LSTM)
* Gated Recurrent Units (GRU)

These can be layered, combined, or attached to create any number of different `Graph` architectures. Or you can build your own from scratch.

Borrowing from the popular [NEAT](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf) (NeuroEvoltuion of Augmenting Topologies) algorithm, Radiate offers a way to evolve the graph architecture itself:

* `GraphCrossover` - Crossover two `Graph`s similar (almost identical) to the NEAT algorithm.
* `GraphMutator` - Mutate a `Graph` by adding different types of `Node`s to the graph.
* `NodeMutator` - Mutate a `Node` by editing its internal (its `Allele`) properties.
* `NodeCrossover` - Crossover two `Node`s by swapping their internal properties.

## Trees

Trees use a very similar pattern to the `Graph` but are more simple in nature. 
The `Architect` for trees, grows a tree given a desired starting minimum depth.

`Tree`s can be evolved using the:

* `TreeCrossover` - Crossover two subtrees of a `Tree`.
* `NodeMutator` - Mutate a `Node` by editing its internal (its `Allele`) properties.
* `NodeCrossover` - Crossover two `Node`s by swapping their internal properties.
