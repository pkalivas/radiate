# Release Notes

---

## v1.2.20 - py 0.0.10

- 2025-12-15
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.20)

Adding radiate-ui crate, bug fixes, & speed improvements. 

I split up some functionality into a new crate radiate-utils and have added a new feature radiate-ui for a tui user interface through [ratitui](https://ratatui.rs). Some small bug fixes, code simplifications, and some nice little speed improvements. 

**[Full Changelog](https://github.com/pkalivas/radiate/compare/v1.2.19...v1.2.20)**

## v1.2.19 - py 0.0.9

- 2025-11-11
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.19)

Adding support for experimental [PGM](https://en.wikipedia.org/wiki/Graphical_model) or Probabilistic graphical models through the GP feature (crate).

Major cleanup or unused code and massive graph performance improvements through the use of [smallvec](https://docs.rs/smallvec/latest/smallvec/) as connections instead of BTreeSets. 

Improving eventing system through cleaner code and removing redundant events.

Introducing radiate-error (RadiateError) into the core crates ad requiring its usage in certain traits (Problem mainly). We also use this error type in py-radiate and allow it to bubble up into python's type system too. 

Brining metrics to the forefront in python. 

**[Full Changelog](https://github.com/pkalivas/radiate/compare/v1.2.18...v1.2.19)**

## v1.2.18 - py 0.0.8

- 2025-09-27
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.18)

Fixing subtle bug in recurrent graphs where a random seed wasn't being respected, leading to non-deterministic behavior in some cases. This fix ensures that all random operations within recurrent graphs are consistent and reproducible when a seed is provided. 

Added three new types of graphs:

- **LSTM** (Long Short-Term Memory) Graphs: These are a type of recurrent neural network (RNN) that can learn long-term dependencies.
- **GRU** (Gated Recurrent Unit) Graphs: Similar to LSTMs, GRUs are a type of RNN that are simpler and often more efficient.
- **Mesh** Graphs: Graphs structured in a mesh topology.

## v1.2.17 - py 0.0.7

- 2025-09-04
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.17)

In response to github issue [#23](https://github.com/pkalivas/radiate/issues/23).

Ensuring that FloatGenes/IntGene<T>'s respect their bounds during mutation and crossover. This was a bug where mutated or crossovered genes could exceed their defined bounds, which could lead to invalid individuals in the population. This fix ensures that all FloatGenes/IntGene<T>'s remain within their specified bounds after any genetic operation. Also some optimizations and code cleanup for py-radiate. Large additions to tests.

Also adding new mutator: `JitterMutator` for FloatGenes. This mutator adds a small random value (jitter) to each gene, controlled by a `magnitude` parameter.

## v1.2.16

- 2025-08-19
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.16)

In response to github issue [#22](https://github.com/pkalivas/radiate/issues/22).

Adding support for batch fitness functions and batch engine problems through a new trait (BatchFitnessFn). Some small cleanup on other fitness functions and some chromosome operators.

## v1.2.15 - py 0.0.6

- 2025-08-10
- [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.15)

Adding Novelty Search to python and refactoring engine building across the rust/python bridge. Improving python's speed. Adding type checking to python and upgrading python package to >= python 3.12 to support new python generics. Improving docs to reference new functionality.

New alters:

  * EdgeRecombinationCrossover for PermutationGenes 
  * PolynomialMutator for chromosomes with FloatGenes

Added code path in alters for dynamic mutation/crossover rates. This is in early dev, but an be seen in PolynomialMutator.


## v1.2.14 - py 0.0.4

-  2025-07-05
-  [Release](https://github.com/pkalivas/radiate/releases/tag/v1.2.14)

Added support for novelty search, fitness-based novelty, and combined novelty and fitness search. Improved documentation and examples. Improved traits for `Engine` and introduced one for `FitnessFn`. Bug fixes for pareto fronts and engine iterators.