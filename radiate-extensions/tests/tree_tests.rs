#[cfg(test)]
mod test {

    // use radiate::*;
    use radiate_extensions::*;

    #[test]
    fn test_tree_build_with_depth_works() {
        let factory = NodeFactory::<f32>::regression(2);
        let architect = Architect::<Tree<f32>, f32>::new(&factory);

        let graph = architect.tree(3);

        for node in graph.get_nodes() {
            println!("{:?} ---- {:?}", node, node.arity);
        }
    }

    #[test]
    fn test_tree_build_with_depth_and_codex_works() {
        // let factory = NodeFactory::<f32>::regression(1).gates(vec![op::add(), op::sub(), op::mul()]);

        // let tree_codex = TreeCodex::new(3, &factory);
    }
}
