use radiate::prelude::*;

fn main() {
    // --8<-- [start:build_tree]
    // create a simple tree:
    //              42
    //           /  |   \
    //          1   2    3
    //             / \
    //            3   4
    let tree: Tree<i32> = Tree::new(
        TreeNode::new(42)
            .attach(TreeNode::new(1))
            .attach(
                TreeNode::new(2)
                    .attach(TreeNode::new(3))
                    .attach(TreeNode::new(4)),
            )
            .attach(TreeNode::new(3)),
    );

    // The tree can be evaluated with a function that takes a vector of inputs
    // This creates a `Tree` that looks like:
    //      +
    //    /   \
    //   *     +
    //  / \   / \
    // 2  3  2   x
    //
    // Where `x` is the first variable in the input.
    // This can also be thought of (and is functionally equivalent) as:
    //
    // f(x) = (2 * 3) + (2 + x)
    //
    let root = TreeNode::new(Op::add())
        .attach(
            TreeNode::new(Op::mul())
                .attach(TreeNode::new(Op::constant(2.0)))
                .attach(TreeNode::new(Op::constant(3.0))),
        )
        .attach(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::constant(2.0)))
                .attach(TreeNode::new(Op::var(0))),
        );

    // And the result of evaluating this tree with an input of `1` would be:
    let result = root.eval(&vec![1_f32]);
    assert_eq!(result, 9.0);
    // --8<-- [end:build_tree]

    // --8<-- [start:tree_codec]
    let store = vec![
        (NodeType::Root, vec![Op::add(), Op::sub()]),
        (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
        (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
    ];

    // Create a single rooted tree codec with a starting (minimum) depth of 3
    let codec = TreeCodec::single(3, store.clone());
    let genotype: Genotype<TreeChromosome<Op<f32>>> = codec.encode();
    let tree: Tree<Op<f32>> = codec.decode(&genotype);

    // Create a multi-rooted tree codec with a starting (minimum) depth of 3 and 2 trees
    let codec = TreeCodec::multi_root(3, 2, store);
    let genotype: Genotype<TreeChromosome<Op<f32>>> = codec.encode();
    // multi-rooted codec decodes to a Vec of Trees
    // one for each root in the genotype
    let trees: Vec<Tree<Op<f32>>> = codec.decode(&genotype);
    // --8<-- [end:tree_codec]

    // --8<-- [start:hoist_mutator]
    let mutator = HoistMutator::new(0.1);
    // --8<-- [end:hoist_mutator]

    // --8<-- [start:tree_crossover]
    let mutator = TreeCrossover::new(0.1);
    // --8<-- [end:tree_crossover]
}
