#[cfg(test)]
mod test {

    use radiate::*;
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
}