#[cfg(test)]
mod tests {

    use radiate::*;
    use radiate_gp::{Direction, Graph, NodeType};

    #[test]
    fn test_simple_graph() {
        let mut graph = Graph::<i32>::default();

        let idx_one = graph.insert(NodeType::Input, 0);
        let idx_two = graph.insert(NodeType::Vertex, 1);
        let idx_three = graph.insert(NodeType::Output, 2);

        graph.attach(idx_one, idx_two).attach(idx_two, idx_three);

        assert_eq!(graph.len(), 3);

        assert!(graph.is_valid());
        assert!(graph[0].is_valid());
        assert!(graph[1].is_valid());
        assert!(graph[2].is_valid());

        assert_eq!(graph[0].incoming().len(), 0);
        assert_eq!(graph[0].outgoing().len(), 1);
        assert_eq!(graph[1].incoming().len(), 1);
        assert_eq!(graph[1].outgoing().len(), 1);
        assert_eq!(graph[2].incoming().len(), 1);
        assert_eq!(graph[2].outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles() {
        let mut graph = Graph::<i32>::default();

        graph.insert(NodeType::Input, 0);
        graph.insert(NodeType::Vertex, 1);
        graph.insert(NodeType::Vertex, 2);
        graph.insert(NodeType::Output, 3);

        graph.attach(0, 1).attach(1, 2).attach(2, 1).attach(2, 3);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph[0].is_valid());
        assert!(graph[1].is_valid());
        assert!(graph[2].is_valid());
        assert!(graph[3].is_valid());

        assert_eq!(graph[0].incoming().len(), 0);
        assert_eq!(graph[0].outgoing().len(), 1);
        assert_eq!(graph[1].incoming().len(), 2);
        assert_eq!(graph[1].outgoing().len(), 1);
        assert_eq!(graph[2].incoming().len(), 1);
        assert_eq!(graph[2].outgoing().len(), 2);
        assert_eq!(graph[3].incoming().len(), 1);
        assert_eq!(graph[3].outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles_and_recurrent_nodes() {
        let mut graph = Graph::<i32>::default();

        graph.insert(NodeType::Input, 0);
        graph.insert(NodeType::Vertex, 1);
        graph.insert(NodeType::Vertex, 2);
        graph.insert(NodeType::Output, 3);

        graph
            .attach(0, 1)
            .attach(1, 2)
            .attach(2, 1)
            .attach(2, 3)
            .attach(3, 1);

        graph.set_cycles(vec![]);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph[0].is_valid());
        assert!(graph[1].is_valid());
        assert!(graph[2].is_valid());
        assert!(graph[3].is_valid());

        assert_eq!(graph[0].incoming().len(), 0);
        assert_eq!(graph[0].outgoing().len(), 1);
        assert_eq!(graph[1].incoming().len(), 3);
        assert_eq!(graph[1].outgoing().len(), 1);
        assert_eq!(graph[2].incoming().len(), 1);
        assert_eq!(graph[2].outgoing().len(), 2);
        assert_eq!(graph[3].incoming().len(), 1);
        assert_eq!(graph[3].outgoing().len(), 1);

        assert_eq!(graph[0].direction(), Direction::Forward);
        assert_eq!(graph[1].direction(), Direction::Backward);
        assert_eq!(graph[2].direction(), Direction::Backward);
        assert_eq!(graph[3].direction(), Direction::Backward);
    }
}
