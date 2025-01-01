#[cfg(test)]
mod tests {
    use radiate::*;
    use radiate_extensions::{
        collections::{GraphBuilder, GraphCodex, GraphReducer},
        Direction, Graph, NodeType, Op,
    };

    #[test]
    fn test_simple_graph() {
        let mut graph = Graph::<i32>::default();

        graph.add(NodeType::Input, 0);
        graph.add(NodeType::Vertex, 1);
        graph.add(NodeType::Output, 2);

        graph.attach(0, 1).attach(1, 2);

        println!("{:?}", graph);

        assert_eq!(graph.len(), 3);

        assert!(graph.is_valid());
        assert!(graph.get(0).is_valid());
        assert!(graph.get(1).is_valid());
        assert!(graph.get(2).is_valid());

        assert_eq!(graph.get(0).incoming().len(), 0);
        assert_eq!(graph.get(0).outgoing().len(), 1);
        assert_eq!(graph.get(1).incoming().len(), 1);
        assert_eq!(graph.get(1).outgoing().len(), 1);
        assert_eq!(graph.get(2).incoming().len(), 1);
        assert_eq!(graph.get(2).outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles() {
        let mut graph = Graph::<i32>::default();

        graph.add(NodeType::Input, 0);
        graph.add(NodeType::Vertex, 1);
        graph.add(NodeType::Vertex, 2);
        graph.add(NodeType::Output, 3);

        graph.attach(0, 1).attach(1, 2).attach(2, 1).attach(2, 3);

        println!("{:?}", graph);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph.get(0).is_valid());
        assert!(graph.get(1).is_valid());
        assert!(graph.get(2).is_valid());
        assert!(graph.get(3).is_valid());

        assert_eq!(graph.get(0).incoming().len(), 0);
        assert_eq!(graph.get(0).outgoing().len(), 1);
        assert_eq!(graph.get(1).incoming().len(), 2);
        assert_eq!(graph.get(1).outgoing().len(), 1);
        assert_eq!(graph.get(2).incoming().len(), 1);
        assert_eq!(graph.get(2).outgoing().len(), 2);
        assert_eq!(graph.get(3).incoming().len(), 1);
        assert_eq!(graph.get(3).outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles_and_recurrent_nodes() {
        let mut graph = Graph::<i32>::default();

        graph.add(NodeType::Input, 0);
        graph.add(NodeType::Vertex, 1);
        graph.add(NodeType::Vertex, 2);
        graph.add(NodeType::Output, 3);

        graph
            .attach(0, 1)
            .attach(1, 2)
            .attach(2, 1)
            .attach(2, 3)
            .attach(3, 1);

        println!("{:?}", graph);

        graph.set_cycles(vec![]);

        println!("{:?}", graph);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph.get(0).is_valid());
        assert!(graph.get(1).is_valid());
        assert!(graph.get(2).is_valid());
        assert!(graph.get(3).is_valid());

        assert_eq!(graph.get(0).incoming().len(), 0);
        assert_eq!(graph.get(0).outgoing().len(), 1);
        assert_eq!(graph.get(1).incoming().len(), 3);
        assert_eq!(graph.get(1).outgoing().len(), 1);
        assert_eq!(graph.get(2).incoming().len(), 1);
        assert_eq!(graph.get(2).outgoing().len(), 2);
        assert_eq!(graph.get(3).incoming().len(), 1);
        assert_eq!(graph.get(3).outgoing().len(), 1);

        assert_eq!(graph.get(0).direction(), Direction::Forward);
        assert_eq!(graph.get(1).direction(), Direction::Backward);
        assert_eq!(graph.get(2).direction(), Direction::Backward);
        assert_eq!(graph.get(3).direction(), Direction::Backward);
    }

    #[test]
    fn test_graph() {
        let graph = GraphBuilder::<Op<f32>>::regression(2).cyclic(2, 2);

        println!("{:?}", graph);
    }

    #[test]
    fn test_reducer() {
        let graph_codex =
            GraphCodex::regression(2, 2).set_nodes(|arc, _| arc.weighted_acyclic(2, 2));

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
}
