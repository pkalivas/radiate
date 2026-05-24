# --8<-- [start:build_tree]
import radiate as rd

codec = rd.TreeCodec(
    shape=(2, 1),  # 2 inputs, 1 output
    min_depth=3,  # starting depth of the tree (default 3)
    max_size=30,  # max number of nodes (default 30)
    vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul(), rd.Op.div()],
    leaf=[rd.Op.var(0), rd.Op.var(1)],
)

# decode a (randomly initialized) genotype into a Tree
tree = codec.decode(codec.encode())

result = tree.eval([1.0, 2.0])  # list[float], one value per output
# --8<-- [end:build_tree]

# --8<-- [start:tree_codec_detailed]
import radiate as rd

# Create a tree codec with a starting (minimum) depth of 3
codec = rd.TreeCodec(
    # Shape is only necessary for multi-rooted trees,
    # it specifies the number of trees and the
    # number of inputs to each tree.
    # For single rooted trees this will be ignored.
    #
    # Default: shape=(1, 1) - single rooted tree with 1 input variable
    shape=(2, 1),
    # The minimum depth of the tree. This is the depth
    # of the initial tree that will be produced by
    # the codec - evolution will be able
    # to increase the total size of the tree (without regards to depth) up to max_size.
    #
    # Default: min_depth=3
    min_depth=3,
    # The maximum size of the tree. This is the maximum number of nodes that the tree can have.
    max_size=30,
    # The options for the root node of the tree. The root is the last node in the tree
    # to be evaluated and is the node that produces the final output of the tree.
    # This is optional and defaults to a single add operator.
    root=rd.Op.add(),
    vertex=[rd.Op.add(), rd.Op.sub(), rd.Op.mul()],
    leaf=[rd.Op.var(0), rd.Op.var(1)],
)

genotype = codec.encode()
tree = codec.decode(genotype)
# --8<-- [end:tree_codec_detailed]

# --8<-- [start:hoist_mutator]
import radiate as rd

mutator = rd.HoistMutator(rate=0.1)
mutator = rd.Mutate.hoist(0.1)  # Using the dsl syntax for mutators
# --8<-- [end:hoist_mutator]

# --8<-- [start:tree_crossover]
import radiate as rd

crossover = rd.TreeCrossover(rate=0.1)
crossover = rd.Cross.tree(0.1)  # Using the dsl syntax for crossover operators
# --8<-- [end:tree_crossover]
