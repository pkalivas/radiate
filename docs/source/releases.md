# Release Notes

---

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