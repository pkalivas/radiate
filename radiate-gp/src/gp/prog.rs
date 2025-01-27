use crate::{Graph, Tree};

#[derive(Clone)]
pub enum Program<T> {
    Tree(Tree<T>),
    MultiTree(Vec<Tree<T>>),
    Graph(Graph<T>),
}

impl<T> Program<T> {
    pub fn tree(tree: Tree<T>) -> Self {
        Program::Tree(tree)
    }

    pub fn multi_tree(trees: Vec<Tree<T>>) -> Self {
        Program::MultiTree(trees)
    }

    pub fn graph(graph: Graph<T>) -> Self {
        Program::Graph(graph)
    }
}

impl<T> From<Tree<T>> for Program<T> {
    fn from(tree: Tree<T>) -> Self {
        Program::Tree(tree)
    }
}

impl<T> From<Vec<Tree<T>>> for Program<T> {
    fn from(trees: Vec<Tree<T>>) -> Self {
        Program::MultiTree(trees)
    }
}

impl<T> From<Graph<T>> for Program<T> {
    fn from(graph: Graph<T>) -> Self {
        Program::Graph(graph)
    }
}
