#[cfg(test)]
mod tests {

    use radiate_extensions::*;
    use radiate_rust::*;

    #[test]
    fn test_graph() {
        let factory = NodeFactory::new()
            .input_values(vec![1, 2, 3])
            .output_values(vec![4, 5, 6])
            .gate_values(vec![7, 8, 9])
            .aggregate_values(vec![10, 11, 12])
            .weight_values(vec![13, 14, 15]);

        let architect = Architect::<Graph<i32>, i32>::new(&factory);

        let graph = architect
            .build(|arc, builder| builder.one_to_one(&arc.input(2), &arc.output(2)).build());

        for node in graph.get_nodes() {
            println!("{:?}", node);
        }
    }

    #[test]
    fn test_acyclic_graph() {
        let factory = NodeFactory::<f32>::regression(2);
        let architect = Architect::<Graph<f32>, f32>::new(&factory);

        let graph = architect.weighted_cyclic(2, 2, 2);

        for node in graph.get_nodes() {
            println!("{:?}", node);
        }
    }

    #[test]
    fn test_reducer() {
        let factory = NodeFactory::<f32>::regression(2);
        let graph_codex =
            GraphCodex::from_factory(&factory).set_nodes(|arc, _| arc.weighted_acyclic(2, 2));

        let genotype = graph_codex.encode();
        let decoded = graph_codex.decode(&genotype);

        for chromosome in genotype.iter() {
            for gene in chromosome.iter() {
                println!("{:?}", gene);
            }
        }

        let inputs = vec![1.0, 2.0];
        let input_two = vec![3.0, 4.0];
        let mut reducer = GraphReducer::new(&decoded);
        let outputs = reducer.reduce(&inputs);

        println!("{:?}", outputs);

        let output_two = reducer.reduce(&input_two);

        println!("{:?}", output_two);
    }

    #[test]
    fn graph_can_modify() {
        let factory = NodeFactory::<f32>::regression(2);
        let graph_codex =
            GraphCodex::from_factory(&factory).set_nodes(|arc, _| arc.weighted_acyclic(2, 2));

        let factory2 = NodeFactory::<f32>::regression(2);
        let modifier =
            GraphMutator::<f32>::new(factory2, vec![NodeMutate::Forward(NodeType::Weight, 0.5)]);

        let genotype = graph_codex.encode();
        let decoded = graph_codex.decode(&genotype);

        for node in decoded.iter() {
            println!("{:?}", node);
        }

        println!("\nModifing graph...\n");

        if let Some(modified) = modifier.insert_forward_node(&decoded.nodes, &NodeType::Weight) {
            for node in modified.iter() {
                println!("{:?}", node);
            }
        }
    }
}
