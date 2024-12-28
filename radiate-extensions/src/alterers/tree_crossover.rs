use crate::expr::Expr;
use crate::node::Node;
use crate::schema::collection_type::CollectionType;
use crate::{NodeChrom, NodeChromosome, TreeNode};
use radiate::alter::AlterType;
use radiate::{random_provider, Alter, Chromosome, Valid};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use uuid::Uuid;

// struct TreeNode<T> {
//     node: Node<T>,
//     children: Vec<TreeNode<T>>,
// }

pub struct TreeCrossover {
    pub rate: f32,
    pub max_height: usize,
}

impl TreeCrossover {
    pub fn new(rate: f32, max_height: usize) -> Self {
        Self { rate, max_height }
    }

    fn level<T>(index: usize, nodes: &[Node<T>]) -> usize {
        nodes[index]
            .incoming
            .iter()
            .map(|i| TreeCrossover::level(*i, nodes))
            .max()
            .unwrap_or(0)
            + 1
    }

    fn depth<T>(index: usize, nodes: &[Node<T>]) -> usize {
        nodes[index]
            .outgoing
            .iter()
            .map(|i| TreeCrossover::depth(*i, nodes))
            .max()
            .unwrap_or(0)
            + 1
    }

    fn can_cross<T>(
        &self,
        one: &[Node<T>],
        two: &[Node<T>],
        one_index: usize,
        two_index: usize,
    ) -> bool {
        if one_index < 1 || two_index < 1 {
            return false;
        }

        let one_depth = TreeCrossover::depth(one_index, one);
        let two_depth = TreeCrossover::depth(two_index, two);

        let one_height = TreeCrossover::level(one_index, one);
        let two_height = TreeCrossover::level(two_index, two);

        one_height + two_depth <= self.max_height && two_height + one_depth <= self.max_height
    }
}

impl<T> Alter<NodeChrom<TreeNode<T>>> for TreeCrossover
where
    T: Clone + PartialEq + Default + Debug,
{
    fn name(&self) -> &'static str {
        "Tree Crossover"
    }

    fn rate(&self) -> f32 {
        self.rate
    }

    fn alter_type(&self) -> AlterType {
        AlterType::Crossover
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut NodeChrom<TreeNode<T>>,
        chrom_two: &mut NodeChrom<TreeNode<T>>,
    ) -> i32 {
        let swap_one_index = random_provider::random::<usize>() % chrom_one.len();
        let swap_two_index = random_provider::random::<usize>() % chrom_two.len();

        let one_node = &mut chrom_one.get_genes_mut()[swap_one_index];
        let mut two_node = &mut chrom_two.get_genes_mut()[swap_two_index];

        let one_size = one_node.size();
        let two_size = two_node.size();

        let one_rand_index = random_provider::random::<usize>() % one_size;
        let two_rand_index = random_provider::random::<usize>() % two_size;

        if one_rand_index < 1 || two_rand_index < 1 {
            return 0;
        }

        // if one_size > self.max_height || two_size > self.max_height {
        //     return 0;
        // }

        // let prev_one = one_node.clone();
        // let prev_two = two_node.clone();

        one_node.swap_subtrees(&mut two_node, one_rand_index, two_rand_index);

        // if one_node.size() > self.max_height || two_node.size() > self.max_height {
        //     *one_node = prev_one;
        //     *two_node = prev_two;
        //     return 0;
        // }

        // chrom_one.nodes = new_one_nodes;
        // chrom_two.nodes = new_two_nodes;

        // if !chrom_one.is_valid() || !chrom_two.is_valid() {
        //     panic!("Invalid tree after crossover.");
        // }

        2
    }
}

// impl<T> Alter<NodeChromosome<T>> for TreeCrossover
// where
//     T: Clone + PartialEq + Default + Debug,
// {
//     fn name(&self) -> &'static str {
//         "Tree Crossover"
//     }

//     fn rate(&self) -> f32 {
//         self.rate
//     }

//     fn alter_type(&self) -> AlterType {
//         AlterType::Crossover
//     }

//     #[inline]
//     fn cross_chromosomes(
//         &self,
//         chrom_one: &mut NodeChromosome<T>,
//         chrom_two: &mut NodeChromosome<T>,
//     ) -> i32 {
//         let swap_one_index = random_provider::random::<usize>() % chrom_one.len();
//         let swap_two_index = random_provider::random::<usize>() % chrom_two.len();

//         if !self.can_cross(
//             chrom_one.get_genes(),
//             chrom_two.get_genes(),
//             swap_one_index,
//             swap_two_index,
//         ) {
//             return 0;
//         }

//         for node in chrom_one.iter_mut() {
//             node.id = Uuid::new_v4();
//         }

//         for node in chrom_two.iter_mut() {
//             node.id = Uuid::new_v4();
//         }

//         let mut tree_1 = to_tree(chrom_one.get_genes());
//         let mut tree_2 = to_tree(chrom_two.get_genes());

//         swap_trees(&mut tree_1, &mut tree_2, swap_one_index, swap_two_index);

//         let new_one_nodes = tree_to_nodes(&tree_1);
//         let new_two_nodes = tree_to_nodes(&tree_2);

//         chrom_one.nodes = new_one_nodes;
//         chrom_two.nodes = new_two_nodes;

//         if !chrom_one.is_valid() || !chrom_two.is_valid() {
//             panic!("Invalid tree after crossover.");
//         }

//         2
//     }
// }

// fn to_tree<T>(nodes: &[Node<T>]) -> TreeNode<T>
// where
//     T: Clone + PartialEq + Default + Debug,
// {
//     let mut visited = HashSet::new();
//     to_tree_recursive(nodes, 0, &mut visited)
// }

// fn to_tree_recursive<T>(
//     nodes: &[Node<T>],
//     current_index: usize,
//     visited: &mut HashSet<usize>,
// ) -> TreeNode<T>
// where
//     T: Clone + PartialEq + Default + Debug,
// {
//     if !visited.insert(current_index) {
//         panic!(
//             "Cycle detected in tree conversion at index {}",
//             current_index
//         );
//     }

//     let node = &nodes[current_index];
//     let mut tree_node = TreeNode {
//         node: node.clone(),
//         children: Vec::new(),
//     };

//     // Process children in sorted order for consistency
//     let mut child_indices: Vec<_> = node.outgoing.iter().copied().collect();
//     child_indices.sort_unstable();

//     for child_index in child_indices {
//         if child_index >= nodes.len() {
//             panic!(
//                 "Invalid child index {} for node {}",
//                 child_index, current_index
//             );
//         }
//         if !visited.contains(&child_index) {
//             tree_node
//                 .children
//                 .push(to_tree_recursive(nodes, child_index, visited));
//         }
//     }

//     tree_node
// }

// fn get_node_queue<T>(root: &TreeNode<T>) -> Vec<&TreeNode<T>> {
//     let mut queue = Vec::new();
//     let mut stack = Vec::new();

//     queue.push(root);

//     while let Some(node) = queue.pop() {
//         for child in node.children.iter() {
//             queue.push(child);
//         }

//         stack.push(node);
//     }

//     stack
// }

// fn tree_to_nodes<T>(root: &TreeNode<T>) -> Vec<Node<T>>
// where
//     T: Clone + PartialEq + Default + Debug,
// {
//     let node_map = get_node_queue(root);

//     let mut node_id_index_map = HashMap::new();
//     let mut tree_node_id_node_map = HashMap::new();
//     let mut nodes = Vec::new();

//     for (index, node) in node_map.iter().enumerate() {
//         node_id_index_map.insert(node.node.id, index);
//         tree_node_id_node_map.insert(node.node.id, node);
//         let mut new_node = Node::new(index, node.node.node_type, node.node.value.clone());
//         new_node.collection_type = Some(CollectionType::Tree);
//         nodes.push(new_node);
//     }

//     for (id, index) in node_id_index_map.iter() {
//         let tree_node = tree_node_id_node_map.get(id).unwrap();
//         for child in tree_node.children.iter() {
//             let child_index = node_id_index_map.get(&child.node.id).unwrap();
//             nodes[*index].outgoing.insert(*child_index);
//             nodes[*child_index].incoming.insert(*index);
//         }
//     }

//     nodes
// }

// fn swap_trees<T>(one: &mut TreeNode<T>, two: &mut TreeNode<T>, one_idx: usize, two_idx: usize) {
//     // Get paths to target nodes
//     let one_path = get_path_to_node(one, one_idx);
//     let two_path = get_path_to_node(two, two_idx);

//     if one_path.len() < 2 || two_path.len() < 2 {
//         return;
//     }

//     // Navigate to the nodes using indices
//     let mut one_current = one;
//     let mut two_current = two;

//     // Follow paths except for last index (which is the child we want to swap)
//     for &idx in one_path.iter().take(one_path.len() - 1) {
//         one_current = &mut one_current.children[idx];
//     }

//     for &idx in two_path.iter().take(two_path.len() - 1) {
//         two_current = &mut two_current.children[idx];
//     }

//     // Get final child indices
//     let one_child_idx = *one_path.last().unwrap();
//     let two_child_idx = *two_path.last().unwrap();

//     // Perform the swap
//     std::mem::swap(
//         &mut one_current.children[one_child_idx],
//         &mut two_current.children[two_child_idx],
//     );
// }

// fn get_path_to_node<T>(root: &TreeNode<T>, target_idx: usize) -> Vec<usize> {
//     let mut path = Vec::new();
//     let mut current_idx = 0;
//     find_path_recursive(root, target_idx, &mut current_idx, &mut path);
//     path
// }

// fn find_path_recursive<T>(
//     node: &TreeNode<T>,
//     target_idx: usize,
//     current_idx: &mut usize,
//     path: &mut Vec<usize>,
// ) -> bool {
//     if *current_idx == target_idx {
//         return true;
//     }

//     for (child_idx, _) in node.children.iter().enumerate() {
//         path.push(child_idx);
//         *current_idx += 1;
//         if find_path_recursive(&node.children[child_idx], target_idx, current_idx, path) {
//             return true;
//         }
//         path.pop();
//     }

//     false
// }
