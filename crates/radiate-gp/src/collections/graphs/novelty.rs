use super::Graph;
use crate::{GraphChromosome, GraphNode, Node, NodeType};
use radiate_core::{Diversity, Genotype, fitness::Novelty};
use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hasher},
};

#[derive(Debug, Clone)]
pub struct GraphTopologyDescriptor {
    pub node_count: f32,
    pub edge_count: f32,
    pub avg_degree: f32,
    pub density: f32,
    pub clustering_coefficient: f32,
    pub avg_path_length: f32,
    pub connected_components: f32,
    pub max_degree: f32,
    pub diameter: f32,
    pub avg_betweenness: f32,
}

#[derive(Debug, Clone)]
pub struct GraphArchitectureDescriptor {
    pub layer_count: f32,
    pub input_output_ratio: f32,
    pub hidden_density: f32,
    pub skip_connections: f32,
    pub fan_out_ratio: f32,
    pub fan_in_ratio: f32,
    pub activation_types: f32,
    pub modularity: f32,
    pub centrality_std: f32,
    pub avg_path_length: f32,
}

#[derive(Clone)]
pub struct GraphTopologyNovelty;

impl GraphTopologyNovelty {
    pub fn extract_topology_metrics<G: AsRef<[GraphNode<T>]>, T>(
        &self,
        graph: &G,
    ) -> GraphTopologyDescriptor {
        let graph = graph.as_ref();
        let node_count = graph.len() as f32;
        let mut edge_count = 0.0;
        let mut total_degree = 0.0;
        let mut max_degree = 0.0;
        let mut degrees = Vec::new();
        let mut adjacency_matrix = vec![vec![false; graph.len()]; graph.len()];

        // Build adjacency matrix and calculate basic metrics
        for (i, node) in graph.iter().enumerate() {
            let degree = node.incoming().len() + node.outgoing().len();
            total_degree += degree as f32;
            max_degree = f32::max(max_degree, degree as f32);
            degrees.push(degree);

            for &outgoing in node.outgoing() {
                if outgoing < graph.len() {
                    adjacency_matrix[i][outgoing] = true;
                    edge_count += 1.0;
                }
            }
        }

        let avg_degree = if node_count > 0.0 {
            total_degree / node_count
        } else {
            0.0
        };
        let density = if node_count > 1.0 {
            edge_count / (node_count * (node_count - 1.0))
        } else {
            0.0
        };

        let clustering_coefficient = Self::calculate_clustering_coefficient(&adjacency_matrix);
        let (avg_path_length, diameter) = Self::calculate_path_metrics(&adjacency_matrix);
        let connected_components = Self::calculate_connected_components(&adjacency_matrix) as f32;
        let avg_betweenness = Self::calculate_average_betweenness(&adjacency_matrix);

        GraphTopologyDescriptor {
            node_count,
            edge_count,
            avg_degree,
            density,
            clustering_coefficient,
            avg_path_length,
            connected_components,
            max_degree,
            diameter,
            avg_betweenness,
        }
    }

    pub fn distance(&self, a: &GraphTopologyDescriptor, b: &GraphTopologyDescriptor) -> f32 {
        let mut total_distance = 0.0;

        let a_norm = [
            a.node_count,
            a.edge_count,
            a.avg_degree,
            a.density,
            a.clustering_coefficient,
            a.avg_path_length,
            a.connected_components,
            a.max_degree,
            a.diameter,
            a.avg_betweenness,
        ];
        let b_norm = [
            b.node_count,
            b.edge_count,
            b.avg_degree,
            b.density,
            b.clustering_coefficient,
            b.avg_path_length,
            b.connected_components,
            b.max_degree,
            b.diameter,
            b.avg_betweenness,
        ];

        for (a_val, b_val) in a_norm.iter().zip(b_norm.iter()) {
            total_distance += (a_val - b_val).abs();
        }

        total_distance
    }

    fn calculate_clustering_coefficient(adjacency: &[Vec<bool>]) -> f32 {
        let mut total_clustering = 0.0;
        let mut valid_nodes = 0;

        for i in 0..adjacency.len() {
            let mut neighbors = Vec::new();
            for j in 0..adjacency.len() {
                if i != j && (adjacency[i][j] || adjacency[j][i]) {
                    neighbors.push(j);
                }
            }

            if neighbors.len() >= 2 {
                let mut triangles = 0;
                let mut possible_triangles = 0;

                for &n1 in &neighbors {
                    for &n2 in &neighbors {
                        if n1 < n2 {
                            possible_triangles += 1;
                            if adjacency[n1][n2] || adjacency[n2][n1] {
                                triangles += 1;
                            }
                        }
                    }
                }

                if possible_triangles > 0 {
                    total_clustering += triangles as f32 / possible_triangles as f32;
                    valid_nodes += 1;
                }
            }
        }

        if valid_nodes > 0 {
            total_clustering / valid_nodes as f32
        } else {
            0.0
        }
    }

    fn calculate_path_metrics(adjacency: &[Vec<bool>]) -> (f32, f32) {
        let n = adjacency.len();
        if n == 0 {
            return (0.0, 0.0);
        }

        let mut distances = vec![vec![f32::INFINITY; n]; n];

        // Initialize distances
        for i in 0..n {
            distances[i][i] = 0.0;
            for j in 0..n {
                if adjacency[i][j] {
                    distances[i][j] = 1.0;
                }
            }
        }

        // Floyd-Warshall algorithm
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    if distances[i][k] + distances[k][j] < distances[i][j] {
                        distances[i][j] = distances[i][k] + distances[k][j];
                    }
                }
            }
        }

        // Calculate average path length and diameter
        let mut total_path_length = 0.0;
        let mut path_count = 0;
        let mut max_path = 0.0;

        for i in 0..n {
            for j in 0..n {
                if i != j && distances[i][j] < f32::INFINITY {
                    total_path_length += distances[i][j];
                    path_count += 1;
                    max_path = f32::max(max_path, distances[i][j]);
                }
            }
        }

        let avg_path_length = if path_count > 0 {
            total_path_length / path_count as f32
        } else {
            0.0
        };
        (avg_path_length, max_path)
    }

    fn calculate_connected_components(adjacency: &[Vec<bool>]) -> usize {
        let n = adjacency.len();
        if n == 0 {
            return 0;
        }

        let mut visited = vec![false; n];
        let mut components = 0;

        for i in 0..n {
            if !visited[i] {
                Self::dfs(i, adjacency, &mut visited);
                components += 1;
            }
        }

        components
    }

    fn dfs(node: usize, adjacency: &[Vec<bool>], visited: &mut [bool]) {
        visited[node] = true;
        for (neighbor, &connected) in adjacency[node].iter().enumerate() {
            if connected && !visited[neighbor] {
                Self::dfs(neighbor, adjacency, visited);
            }
        }
    }

    fn calculate_average_betweenness(adjacency: &[Vec<bool>]) -> f32 {
        let n = adjacency.len();
        if n == 0 {
            return 0.0;
        }

        let mut betweenness = vec![0.0; n];

        for s in 0..n {
            for t in 0..n {
                if s != t {
                    let paths = Self::find_shortest_paths(s, t, adjacency);
                    if !paths.is_empty() {
                        let path_count = paths.len() as f32;
                        for path in paths {
                            for &node in &path[1..path.len() - 1] {
                                betweenness[node] += 1.0 / path_count;
                            }
                        }
                    }
                }
            }
        }

        betweenness.iter().sum::<f32>() / n as f32
    }

    fn find_shortest_paths(start: usize, end: usize, adjacency: &[Vec<bool>]) -> Vec<Vec<usize>> {
        let mut paths: Vec<Vec<usize>> = Vec::new();
        let mut queue = vec![(start, vec![start])];
        let mut visited = HashSet::new();

        while let Some((current, path)) = queue.pop() {
            if current == end {
                if paths.is_empty() || path.len() == paths[0].len() {
                    paths.push(path);
                } else if path.len() < paths[0].len() {
                    paths.clear();
                    paths.push(path);
                }
                continue;
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            for (neighbor, &connected) in adjacency[current].iter().enumerate() {
                if connected && !path.contains(&neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push((neighbor, new_path));
                }
            }
        }

        paths
    }
}

impl<T> Diversity<GraphChromosome<T>> for GraphTopologyNovelty
where
    T: Clone + PartialEq,
{
    fn measure(
        &self,
        geno_one: &Genotype<GraphChromosome<T>>,
        geno_two: &Genotype<GraphChromosome<T>>,
    ) -> f32 {
        let mut total_distance = 0.0;
        for (one_chrom, two_chrom) in geno_one.iter().zip(geno_two.iter()) {
            let one_desc = self.extract_topology_metrics(one_chrom);
            let two_desc = self.extract_topology_metrics(two_chrom);

            total_distance += Self::distance(self, &one_desc, &two_desc);
        }

        total_distance
    }
}

impl<T> Novelty<Graph<T>> for GraphTopologyNovelty {
    fn description(&self, graph: &Graph<T>) -> Vec<f32> {
        let metrics = self.extract_topology_metrics(graph);

        vec![
            metrics.node_count,
            metrics.edge_count,
            metrics.avg_degree,
            metrics.density,
            metrics.clustering_coefficient,
            metrics.avg_path_length,
            metrics.connected_components,
            metrics.max_degree,
            metrics.diameter,
            metrics.avg_betweenness,
        ]
    }
}

#[derive(Clone)]
pub struct GraphArchitectureNovelty;

impl GraphArchitectureNovelty {
    pub fn extract_architecture_metrics<G: AsRef<[GraphNode<T>]>, T>(
        &self,
        graph: &G,
    ) -> GraphArchitectureDescriptor
    where
        T: std::hash::Hash,
    {
        let graph = graph.as_ref();

        let mut input_nodes = Vec::new();
        let mut output_nodes = Vec::new();
        let mut hidden_nodes = Vec::new();

        for (i, node) in graph.iter().enumerate() {
            let node_type = node.node_type();
            match node_type {
                NodeType::Input => input_nodes.push(i),
                NodeType::Output => output_nodes.push(i),
                NodeType::Vertex | NodeType::Edge => hidden_nodes.push(i),
                _ => {}
            }
        }

        let layer_count = self.calculate_layer_count(graph, &input_nodes, &output_nodes);

        let input_output_ratio = if output_nodes.is_empty() {
            1.0
        } else {
            input_nodes.len() as f32 / output_nodes.len() as f32
        };

        let hidden_density = if hidden_nodes.len() > 1 {
            let mut hidden_edges = 0;
            for &node_idx in &hidden_nodes {
                let node = &graph[node_idx];
                for &outgoing in node.outgoing() {
                    if hidden_nodes.contains(&outgoing) {
                        hidden_edges += 1;
                    }
                }
            }
            hidden_edges as f32 / (hidden_nodes.len() * (hidden_nodes.len() - 1)) as f32
        } else {
            0.0
        };

        let skip_connections =
            Self::calculate_skip_connections(graph, &input_nodes, &output_nodes, &hidden_nodes);

        let fan_out_ratio = if graph.len() > 0 {
            graph
                .iter()
                .map(|node| node.outgoing().len())
                .sum::<usize>() as f32
                / graph.len() as f32
        } else {
            0.0
        };

        let fan_in_ratio = if graph.len() > 0 {
            graph
                .iter()
                .map(|node| node.incoming().len())
                .sum::<usize>() as f32
                / graph.len() as f32
        } else {
            0.0
        };

        let activation_types = Self::count_activation_types(graph);
        let modularity = Self::calculate_modularity(graph);
        let centrality_std = Self::calculate_centrality_std(graph);
        let avg_path_length = Self::calculate_avg_path_length(graph);

        GraphArchitectureDescriptor {
            layer_count,
            input_output_ratio,
            hidden_density,
            skip_connections,
            fan_out_ratio,
            fan_in_ratio,
            activation_types,
            modularity,
            centrality_std,
            avg_path_length,
        }
    }

    pub fn distance(
        &self,
        a: &GraphArchitectureDescriptor,
        b: &GraphArchitectureDescriptor,
    ) -> f32 {
        let mut total_distance = 0.0;

        let a_norm = [
            a.layer_count,
            a.input_output_ratio,
            a.hidden_density,
            a.skip_connections,
            a.fan_out_ratio,
            a.fan_in_ratio,
            a.activation_types,
            a.modularity,
            a.centrality_std,
            a.avg_path_length,
        ];
        let b_norm = [
            b.layer_count,
            b.input_output_ratio,
            b.hidden_density,
            b.skip_connections,
            b.fan_out_ratio,
            b.fan_in_ratio,
            b.activation_types,
            b.modularity,
            b.centrality_std,
            b.avg_path_length,
        ];

        for (a_val, b_val) in a_norm.iter().zip(b_norm.iter()) {
            total_distance += (a_val - b_val).abs();
        }

        total_distance
    }

    fn calculate_layer_count<T>(
        &self,
        graph: &[GraphNode<T>],
        input_nodes: &[usize],
        output_nodes: &[usize],
    ) -> f32 {
        let mut max_path_length = 0;

        for &input_node in input_nodes {
            for &output_node in output_nodes {
                if let Some(path_length) =
                    Self::shortest_path_length(graph, input_node, output_node)
                {
                    max_path_length = max_path_length.max(path_length);
                }
            }
        }

        max_path_length as f32
    }

    fn shortest_path_length<T>(graph: &[GraphNode<T>], start: usize, end: usize) -> Option<usize> {
        let mut distances = vec![usize::MAX; graph.len()];
        let mut queue = std::collections::VecDeque::new();

        distances[start] = 0;
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if current == end {
                return Some(distances[current]);
            }

            for &outgoing in graph[current].outgoing() {
                if distances[outgoing] == usize::MAX {
                    distances[outgoing] = distances[current] + 1;
                    queue.push_back(outgoing);
                }
            }
        }

        None
    }

    fn calculate_skip_connections<T>(
        graph: &[GraphNode<T>],
        input_nodes: &[usize],
        output_nodes: &[usize],
        hidden_nodes: &[usize],
    ) -> f32 {
        let mut skip_connections = 0;

        // Count direct input to output connections
        for &input_node in input_nodes {
            for &output_node in output_nodes {
                if graph[input_node].outgoing().contains(&output_node) {
                    skip_connections += 1;
                }
            }
        }

        // Count hidden to output connections
        for &hidden_node in hidden_nodes {
            for &output_node in output_nodes {
                if graph[hidden_node].outgoing().contains(&output_node) {
                    skip_connections += 1;
                }
            }
        }

        skip_connections as f32
    }

    fn count_activation_types<T>(graph: &[GraphNode<T>]) -> f32
    where
        T: std::hash::Hash,
    {
        let mut activation_types = HashSet::new();

        for node in graph.iter() {
            let mut hasher = DefaultHasher::new();
            node.value().hash(&mut hasher);
            let hash = hasher.finish();
            activation_types.insert(hash);
        }

        activation_types.len() as f32
    }

    fn calculate_modularity<T>(graph: &[GraphNode<T>]) -> f32 {
        if graph.len() < 2 {
            return 0.0;
        }

        // Find communities using connected components
        let mut communities = Vec::new();
        let mut visited = vec![false; graph.len()];

        for i in 0..graph.len() {
            if !visited[i] {
                let mut community = Vec::new();
                Self::dfs_community(i, graph, &mut visited, &mut community);
                communities.push(community);
            }
        }

        // If only one community, modularity is 0
        if communities.len() < 2 {
            return 0.0;
        }

        // Calculate total edges
        let total_edges = graph
            .iter()
            .map(|node| node.outgoing().len())
            .sum::<usize>() as f32;

        if total_edges == 0.0 {
            return 0.0;
        }

        // Calculate modularity for each community
        let mut modularity = 0.0;

        for community in communities {
            let mut internal_edges = 0.0;
            let mut community_degree = 0.0;

            for &node_idx in &community {
                let node = &graph[node_idx];
                let node_degree = node.outgoing().len() + node.incoming().len();
                community_degree += node_degree as f32;

                // Count internal edges (avoid double counting)
                for &outgoing in node.outgoing() {
                    if community.contains(&outgoing) && node_idx < outgoing {
                        internal_edges += 1.0;
                    }
                }
            }

            let expected_edges = (community_degree * community_degree) / (4.0 * total_edges);

            modularity += (internal_edges - expected_edges) / total_edges;
        }

        modularity.max(-1.0).min(1.0)
    }

    pub fn dfs_community<T>(
        node: usize,
        graph: &[GraphNode<T>],
        visited: &mut [bool],
        community: &mut Vec<usize>,
    ) {
        visited[node] = true;
        community.push(node);

        for &outgoing in graph[node].outgoing() {
            if !visited[outgoing] {
                Self::dfs_community(outgoing, graph, visited, community);
            }
        }

        for &incoming in graph[node].incoming() {
            if !visited[incoming] {
                Self::dfs_community(incoming, graph, visited, community);
            }
        }
    }

    fn calculate_centrality_std<T>(graph: &[GraphNode<T>]) -> f32 {
        let mut centrality = vec![0.0; graph.len()];

        for s in 0..graph.len() {
            for t in 0..graph.len() {
                if s != t {
                    if let Some(path) = Self::shortest_path(graph, s, t) {
                        for &node in &path[1..path.len() - 1] {
                            centrality[node] += 1.0;
                        }
                    }
                }
            }
        }

        if centrality.is_empty() {
            return 0.0;
        }

        let mean = centrality.iter().sum::<f32>() / centrality.len() as f32;
        let variance =
            centrality.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / centrality.len() as f32;

        variance.sqrt()
    }

    fn shortest_path<T>(graph: &[GraphNode<T>], start: usize, end: usize) -> Option<Vec<usize>> {
        let mut distances = vec![usize::MAX; graph.len()];
        let mut previous = vec![None; graph.len()];
        let mut queue = std::collections::VecDeque::new();

        distances[start] = 0;
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if current == end {
                let mut path = Vec::new();
                let mut current = end;
                while let Some(prev) = previous[current] {
                    path.push(current);
                    current = prev;
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            for &outgoing in graph[current].outgoing() {
                if distances[outgoing] == usize::MAX {
                    distances[outgoing] = distances[current] + 1;
                    previous[outgoing] = Some(current);
                    queue.push_back(outgoing);
                }
            }
        }

        None
    }

    fn calculate_avg_path_length<T>(graph: &[GraphNode<T>]) -> f32 {
        let mut total_length = 0.0;
        let mut path_count = 0;

        for i in 0..graph.len() {
            for j in 0..graph.len() {
                if i != j {
                    if let Some(length) = Self::shortest_path_length(graph, i, j) {
                        total_length += length as f32;
                        path_count += 1;
                    }
                }
            }
        }

        if path_count > 0 {
            total_length / path_count as f32
        } else {
            0.0
        }
    }
}

impl<T> Diversity<GraphChromosome<T>> for GraphArchitectureNovelty
where
    T: Clone + PartialEq + std::hash::Hash,
{
    fn measure(
        &self,
        geno_one: &Genotype<GraphChromosome<T>>,
        geno_two: &Genotype<GraphChromosome<T>>,
    ) -> f32 {
        let mut total_distance = 0.0;
        for (one_chrom, two_chrom) in geno_one.iter().zip(geno_two.iter()) {
            let one_desc = self.extract_architecture_metrics(one_chrom);
            let two_desc = self.extract_architecture_metrics(two_chrom);

            let one = vec![
                one_desc.layer_count,
                one_desc.input_output_ratio,
                one_desc.hidden_density,
                one_desc.skip_connections,
                one_desc.fan_out_ratio,
                one_desc.fan_in_ratio,
                one_desc.activation_types,
                one_desc.modularity,
                one_desc.centrality_std,
                one_desc.avg_path_length,
            ];

            let two_desc = vec![
                two_desc.layer_count,
                two_desc.input_output_ratio,
                two_desc.hidden_density,
                two_desc.skip_connections,
                two_desc.fan_out_ratio,
                two_desc.fan_in_ratio,
                two_desc.activation_types,
                two_desc.modularity,
                two_desc.centrality_std,
                two_desc.avg_path_length,
            ];

            for (a_val, b_val) in one.iter().zip(two_desc.iter()) {
                total_distance += (a_val - b_val).abs();
            }
        }

        total_distance
    }
}

impl<T> Novelty<Graph<T>> for GraphArchitectureNovelty
where
    T: std::hash::Hash,
{
    fn description(&self, graph: &Graph<T>) -> Vec<f32> {
        let desc = self.extract_architecture_metrics(graph);

        vec![
            desc.layer_count,
            desc.input_output_ratio,
            desc.hidden_density,
            desc.skip_connections,
            desc.fan_out_ratio,
            desc.fan_in_ratio,
            desc.activation_types,
            desc.modularity,
            desc.centrality_std,
            desc.avg_path_length,
        ]
    }

    // fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
    //     let mut total_distance = 0.0;

    //     for (a_val, b_val) in a.iter().zip(b.iter()) {
    //         total_distance += (a_val - b_val).abs();
    //     }

    //     total_distance
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, NodeType, Op};

    // Helper function to create different graph topologies
    fn create_star_graph() -> Graph<Op<f32>> {
        let mut graph = Graph::new(vec![]);
        let center = graph.insert(NodeType::Vertex, Op::add());
        let inputs: Vec<usize> = (0..5)
            .map(|i| graph.insert(NodeType::Input, Op::var(i)))
            .collect();
        let output = graph.insert(NodeType::Output, Op::linear());

        for input in inputs {
            graph.attach(input, center);
        }
        graph.attach(center, output);
        graph
    }

    fn create_ring_graph() -> Graph<Op<f32>> {
        let mut graph = Graph::new(vec![]);
        let nodes: Vec<usize> = (0..6)
            .map(|_| graph.insert(NodeType::Vertex, Op::add()))
            .collect();

        for i in 0..nodes.len() {
            let next = (i + 1) % nodes.len();
            graph.attach(nodes[i], nodes[next]);
        }

        let input = graph.insert(NodeType::Input, Op::var(0));
        let output = graph.insert(NodeType::Output, Op::linear());
        graph.attach(input, nodes[0]);
        graph.attach(nodes[nodes.len() - 1], output);
        graph
    }

    fn create_mesh_graph() -> Graph<Op<f32>> {
        let mut graph = Graph::new(vec![]);
        let nodes: Vec<usize> = (0..9)
            .map(|_| graph.insert(NodeType::Vertex, Op::sigmoid()))
            .collect();

        // Create 3x3 mesh
        for i in 0..3 {
            for j in 0..3 {
                let current = i * 3 + j;
                if j < 2 {
                    // Right connection
                    graph.attach(nodes[current], nodes[current + 1]);
                }
                if i < 2 {
                    // Down connection
                    graph.attach(nodes[current], nodes[current + 3]);
                }
            }
        }

        let input = graph.insert(NodeType::Input, Op::var(0));
        let output = graph.insert(NodeType::Output, Op::linear());
        graph.attach(input, nodes[0]);
        graph.attach(nodes[8], output);

        graph
    }

    fn create_deep_network() -> Graph<Op<f32>> {
        let mut graph = Graph::new(vec![]);
        let input1 = graph.insert(NodeType::Input, Op::var(0));
        let input2 = graph.insert(NodeType::Input, Op::var(1));

        let layer1_1 = graph.insert(NodeType::Vertex, Op::add());
        let layer1_2 = graph.insert(NodeType::Vertex, Op::mul());

        let layer2_1 = graph.insert(NodeType::Vertex, Op::sin());
        let layer2_2 = graph.insert(NodeType::Vertex, Op::cos());

        let layer3 = graph.insert(NodeType::Vertex, Op::add());
        let output = graph.insert(NodeType::Output, Op::sigmoid());

        // Input to layer 1
        graph.attach(input1, layer1_1);
        graph.attach(input2, layer1_1);
        graph.attach(input1, layer1_2);
        graph.attach(input2, layer1_2);

        // Layer 1 to layer 2
        graph.attach(layer1_1, layer2_1);
        graph.attach(layer1_2, layer2_2);

        // Layer 2 to layer 3
        graph.attach(layer2_1, layer3);
        graph.attach(layer2_2, layer3);

        // Layer 3 to output
        graph.attach(layer3, output);
        graph
    }

    fn create_skip_connection_network() -> Graph<Op<f32>> {
        let mut graph = Graph::new(vec![]);
        let input1 = graph.insert(NodeType::Input, Op::var(0));
        let input2 = graph.insert(NodeType::Input, Op::var(1));

        let hidden1 = graph.insert(NodeType::Vertex, Op::add());
        let hidden2 = graph.insert(NodeType::Vertex, Op::mul());
        let hidden3 = graph.insert(NodeType::Vertex, Op::sin());

        let output = graph.insert(NodeType::Output, Op::linear());

        // Normal connections
        graph.attach(input1, hidden1);
        graph.attach(input2, hidden1);
        graph.attach(hidden1, hidden2);
        graph.attach(hidden2, hidden3);
        graph.attach(hidden3, output);

        // Skip connections
        graph.attach(input1, output); // Direct input to output
        graph.attach(hidden1, output); // Skip from hidden1 to output

        graph
    }

    // Topology Novelty Tests
    #[test]
    fn test_topology_novelty_basic() {
        let novelty = GraphTopologyNovelty;
        let graph = create_star_graph();

        let descriptor = novelty.extract_topology_metrics(&graph);

        // Basic sanity checks
        assert!(descriptor.node_count > 0.0);
        assert!(descriptor.edge_count > 0.0);
        assert!(descriptor.avg_degree >= 0.0);
        assert!(descriptor.density >= 0.0);
        assert!(descriptor.density <= 1.0);
        assert!(descriptor.clustering_coefficient >= 0.0);
        assert!(descriptor.clustering_coefficient <= 1.0);
        assert!(descriptor.connected_components > 0.0);
        assert!(descriptor.max_degree >= 0.0);
        assert!(descriptor.diameter >= 0.0);
        assert!(descriptor.avg_betweenness >= 0.0);
    }

    #[test]
    fn test_topology_novelty_different_structures() {
        let novelty = GraphTopologyNovelty;

        let star = create_star_graph();
        let ring = create_ring_graph();
        let mesh = create_mesh_graph();

        let star_desc = novelty.extract_topology_metrics(&star);
        let ring_desc = novelty.extract_topology_metrics(&ring);
        let mesh_desc = novelty.extract_topology_metrics(&mesh);

        // Different structures should have different descriptors
        assert_ne!(star_desc.node_count, ring_desc.node_count);
        assert_ne!(star_desc.edge_count, ring_desc.edge_count);
        assert_ne!(mesh_desc.avg_degree, star_desc.avg_degree);

        // Test distance calculation
        let distance = novelty.distance(&star_desc, &ring_desc);
        assert!(distance > 0.0);

        let distance2 = novelty.distance(&star_desc, &mesh_desc);
        assert!(distance2 > 0.0);
    }

    #[test]
    fn test_topology_novelty_empty_graph() {
        let novelty = GraphTopologyNovelty;
        let empty_graph = Graph::<Op<f32>>::default();

        let descriptor = novelty.extract_topology_metrics(&empty_graph);

        assert_eq!(descriptor.node_count, 0.0);
        assert_eq!(descriptor.edge_count, 0.0);
        assert_eq!(descriptor.avg_degree, 0.0);
        assert_eq!(descriptor.density, 0.0);
        assert_eq!(descriptor.clustering_coefficient, 0.0);
        assert_eq!(descriptor.connected_components, 0.0);
        assert_eq!(descriptor.max_degree, 0.0);
        assert_eq!(descriptor.diameter, 0.0);
        assert_eq!(descriptor.avg_betweenness, 0.0);
    }

    #[test]
    fn test_topology_novelty_single_node() {
        let novelty = GraphTopologyNovelty;
        let mut graph = Graph::new(vec![]);
        graph.insert(NodeType::Vertex, Op::add());

        let descriptor = novelty.extract_topology_metrics(&graph);

        assert_eq!(descriptor.node_count, 1.0);
        assert_eq!(descriptor.edge_count, 0.0);
        assert_eq!(descriptor.avg_degree, 0.0);
        assert_eq!(descriptor.density, 0.0);
        assert_eq!(descriptor.connected_components, 1.0);
        assert_eq!(descriptor.max_degree, 0.0);
    }

    // Architecture Novelty Tests
    #[test]
    fn test_architecture_novelty_basic() {
        let novelty = GraphArchitectureNovelty;
        let graph = create_deep_network();

        let descriptor = novelty.extract_architecture_metrics(&graph);

        // Basic sanity checks
        assert!(descriptor.layer_count >= 0.0);
        assert!(descriptor.input_output_ratio > 0.0);
        assert!(descriptor.hidden_density >= 0.0);
        assert!(descriptor.hidden_density <= 1.0);
        assert!(descriptor.skip_connections >= 0.0);
        assert!(descriptor.fan_out_ratio >= 0.0);
        assert!(descriptor.fan_in_ratio >= 0.0);
        assert!(descriptor.activation_types >= 0.0);
        assert!(descriptor.modularity >= -1.0);
        assert!(descriptor.modularity <= 1.0);
        assert!(descriptor.centrality_std >= 0.0);
        assert!(descriptor.avg_path_length >= 0.0);
    }

    #[test]
    fn test_architecture_novelty_skip_connections() {
        let novelty = GraphArchitectureNovelty;

        let normal_network = create_deep_network();
        let skip_network = create_skip_connection_network();

        let normal_desc = novelty.extract_architecture_metrics(&normal_network);
        let skip_desc = novelty.extract_architecture_metrics(&skip_network);

        // Skip connection network should have more skip connections
        assert!(skip_desc.skip_connections > normal_desc.skip_connections);

        // Different architectures should have different descriptors
        assert_ne!(normal_desc.layer_count, skip_desc.layer_count);
        assert_ne!(normal_desc.fan_out_ratio, skip_desc.fan_out_ratio);
    }

    #[test]
    fn test_architecture_novelty_activation_diversity() {
        let novelty = GraphArchitectureNovelty;

        let mut diverse_graph = Graph::new(vec![]);
        let input = diverse_graph.insert(NodeType::Input, Op::var(0));
        let sigmoid_node = diverse_graph.insert(NodeType::Vertex, Op::sigmoid());
        let tanh_node = diverse_graph.insert(NodeType::Vertex, Op::tanh());
        let relu_node = diverse_graph.insert(NodeType::Vertex, Op::relu());
        let linear_node = diverse_graph.insert(NodeType::Vertex, Op::linear());
        let output = diverse_graph.insert(NodeType::Output, Op::linear());

        diverse_graph.attach(input, sigmoid_node);
        diverse_graph.attach(sigmoid_node, tanh_node);
        diverse_graph.attach(tanh_node, relu_node);
        diverse_graph.attach(relu_node, linear_node);
        diverse_graph.attach(linear_node, output);

        let descriptor = novelty.extract_architecture_metrics(&diverse_graph);

        // Should detect multiple activation types
        assert!(descriptor.activation_types >= 3.0);
    }

    #[test]
    fn test_architecture_novelty_empty_graph() {
        let novelty = GraphArchitectureNovelty;
        let empty_graph = Graph::<Op<f32>>::default();

        let descriptor = novelty.extract_architecture_metrics(&empty_graph);

        assert_eq!(descriptor.layer_count, 0.0);
        assert_eq!(descriptor.input_output_ratio, 1.0); // Default when no outputs
        assert_eq!(descriptor.hidden_density, 0.0);
        assert_eq!(descriptor.skip_connections, 0.0);
        assert_eq!(descriptor.fan_out_ratio, 0.0);
        assert_eq!(descriptor.fan_in_ratio, 0.0);
        assert_eq!(descriptor.activation_types, 0.0);
        assert_eq!(descriptor.modularity, 0.0);
        assert_eq!(descriptor.centrality_std, 0.0);
        assert_eq!(descriptor.avg_path_length, 0.0);
    }

    #[test]
    fn test_architecture_novelty_only_inputs() {
        let novelty = GraphArchitectureNovelty;
        let mut graph = Graph::<Op<f32>>::new(vec![]);
        graph.insert(NodeType::Input, Op::var(0));
        graph.insert(NodeType::Input, Op::var(1));

        let descriptor = novelty.extract_architecture_metrics(&graph);

        assert_eq!(descriptor.layer_count, 0.0);
        assert_eq!(descriptor.input_output_ratio, 1.0); // 2 inputs, 0 outputs
        assert_eq!(descriptor.hidden_density, 0.0);
        assert_eq!(descriptor.skip_connections, 0.0);
    }

    #[test]
    fn test_architecture_novelty_only_outputs() {
        let novelty = GraphArchitectureNovelty;
        let mut graph = Graph::new(vec![]);
        graph.insert(NodeType::Output, Op::linear());
        graph.insert(NodeType::Output, Op::sigmoid());

        let descriptor = novelty.extract_architecture_metrics(&graph);

        assert_eq!(descriptor.layer_count, 0.0);
        assert_eq!(descriptor.input_output_ratio, 0.0); // 0 inputs, 2 outputs
        assert_eq!(descriptor.hidden_density, 0.0);
        assert_eq!(descriptor.skip_connections, 0.0);
    }

    #[test]
    fn test_architecture_novelty_self_distance() {
        let novelty = GraphArchitectureNovelty;
        let graph = create_deep_network();
        let descriptor = novelty.extract_architecture_metrics(&graph);

        // Distance to self should be 0
        let self_distance = novelty.distance(&descriptor, &descriptor);
        assert_eq!(self_distance, 0.0);
    }

    #[test]
    fn test_architecture_novelty_different_architectures() {
        let novelty = GraphArchitectureNovelty;

        let deep = create_deep_network();
        let skip = create_skip_connection_network();

        let deep_desc = novelty.extract_architecture_metrics(&deep);
        let skip_desc = novelty.extract_architecture_metrics(&skip);

        // Different architectures should have different descriptors
        assert_ne!(deep_desc.layer_count, skip_desc.layer_count);
        assert_ne!(deep_desc.skip_connections, skip_desc.skip_connections);

        // Test distance calculation
        let distance = novelty.distance(&deep_desc, &skip_desc);
        assert!(distance > 0.0);
    }

    #[test]
    fn test_architecture_novelty_modularity() {
        let novelty = GraphArchitectureNovelty;

        // Create a more clearly modular graph with two distinct communities
        let mut modular_graph = Graph::new(vec![]);

        // Input layer
        let input1 = modular_graph.insert(NodeType::Input, Op::var(0));
        let input2 = modular_graph.insert(NodeType::Input, Op::var(1));

        // Community 1: Processing pathway 1
        let node1 = modular_graph.insert(NodeType::Vertex, Op::add());
        let node2 = modular_graph.insert(NodeType::Vertex, Op::mul());
        let node3 = modular_graph.insert(NodeType::Vertex, Op::sin());

        // Community 2: Processing pathway 2
        let node4 = modular_graph.insert(NodeType::Vertex, Op::cos());
        let node5 = modular_graph.insert(NodeType::Vertex, Op::tanh());
        let node6 = modular_graph.insert(NodeType::Vertex, Op::relu());

        // Output layer
        let output1 = modular_graph.insert(NodeType::Output, Op::linear());
        let output2 = modular_graph.insert(NodeType::Output, Op::sigmoid());

        // Internal connections within Community 1
        modular_graph.attach(node1, node2);
        modular_graph.attach(node2, node3);

        // Internal connections within Community 2
        modular_graph.attach(node4, node5);
        modular_graph.attach(node5, node6);

        // Input connections to communities
        modular_graph.attach(input1, node1); // Input 1 -> Community 1
        modular_graph.attach(input2, node4); // Input 2 -> Community 2

        // Output connections from communities
        modular_graph.attach(node3, output1); // Community 1 -> Output 1
        modular_graph.attach(node6, output2); // Community 2 -> Output 2

        let descriptor = novelty.extract_architecture_metrics(&modular_graph);

        // Should have positive modularity for a clearly modular structure
        assert!(
            descriptor.modularity > 0.0,
            "Modularity was {:.4}, expected > 0.0",
            descriptor.modularity
        );
    }

    #[test]
    fn test_novelty_consistency() {
        let topology_novelty = GraphTopologyNovelty;
        let architecture_novelty = GraphArchitectureNovelty;

        let graph = create_deep_network();

        // Multiple calls should give consistent results
        let desc1 = topology_novelty.description(&graph);
        let desc2 = topology_novelty.description(&graph);

        let arch_desc1 = architecture_novelty.extract_architecture_metrics(&graph);
        let arch_desc2 = architecture_novelty.extract_architecture_metrics(&graph);

        // Descriptors should be identical for same graph
        assert_eq!(architecture_novelty.distance(&arch_desc1, &arch_desc2), 0.0);
    }
}
