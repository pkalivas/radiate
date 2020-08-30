
## Radiate Matrix Tree

This is a pre-built model which is compatible with [Radiate](https://crates.io/crates/radiate), a parallel evolutionary engine.

### Evtree
Is a twist on decision trees where instead of using a certain split criteria like the gini index, each node in the tree has a collection of matrices and uses these matrices to decide which subtree to explore. It is a binary tree and is only good for classification right now. 