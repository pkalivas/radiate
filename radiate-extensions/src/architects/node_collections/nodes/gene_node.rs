// pub enum GeneNode<T> {
//     Graph(GraphNode<T>),
//     Tree(TreeNode<T>),
// }
//
// impl<T> GeneNode<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub fn new_graph(index: usize, value: T) -> Self {
//         GeneNode::Graph(GraphNode::new(index, value))
//     }
//
//     pub fn new_tree(value: T) -> Self {
//         GeneNode::Tree(TreeNode::new(value))
//     }
// }
//
// impl<T> Clone for GeneNode<T>
// where
//     T: Clone,
// {
//     fn clone(&self) -> Self {
//         match self {
//             GeneNode::Graph(node) => GeneNode::Graph(node.clone()),
//             GeneNode::Tree(tree_node) => GeneNode::Tree(tree_node.clone()),
//         }
//     }
// }

// use crate::NodeBehavior;
// use crate::NodeCell;
//
// pub struct GeneNode<T> {
//     index: usize,
//     cell: NodeCell<T>,
//     incoming: HashSet<usize>,
//     outgoing: HashSet<usize>,
// }
//
// impl<T> GeneNode<T> {
//     pub fn new(index: usize, value: T) -> Self {
//         Self {
//             index,
//             cell: NodeCell::new(value),
//             incoming: HashSet::new(),
//             outgoing: HashSet::new(),
//         }
//     }
// }
//
// impl<T> Clone for GeneNode<T>
// where
//     T: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             index: self.index,
//             cell: self.cell.clone(),
//             incoming: self.incoming.clone(),
//             outgoing: self.outgoing.clone(),
//         }
//     }
// }
//
// impl<T> Gene for GeneNode<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     type Allele = T;
//
//     fn allele(&self) -> &Self::Allele {
//         self.cell.value()
//     }
//
//     fn new_instance(&self) -> Self {
//         Self {
//             index: self.index,
//             cell: self.cell.new_instance(),
//             incoming: self.incoming.clone(),
//             outgoing: self.outgoing.clone(),
//         }
//     }
//
//     fn with_allele(&self, allele: &Self::Allele) -> Self {
//         Self {
//             index: self.index,
//             cell: self.cell.clone().with_value(allele.clone()),
//             incoming: self.incoming.clone(),
//             outgoing: self.outgoing.clone(),
//         }
//     }
// }
//
// impl<T> Valid for GeneNode<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     fn is_valid(&self) -> bool {
//         todo!()
//     }
// }
//
// impl<T> PartialEq for GeneNode<T>
// where
//     T: PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.cell == other.cell
//             && self.incoming == other.incoming
//             && self.outgoing == other.outgoing
//     }
// }
