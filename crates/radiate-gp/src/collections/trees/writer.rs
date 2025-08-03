use crate::{Node, ToDot, Tree, TreeNode};
use std::fmt::Display;

impl<T> ToDot for Tree<T>
where
    T: Display,
{
    fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {\n");

        dot_recursive(self.root().unwrap(), 0, &mut dot);

        dot.push_str("}\n");
        dot
    }
}

fn dot_recursive<T: Display>(node: &TreeNode<T>, id: usize, dot: &mut String) -> usize {
    dot.push_str(&format!("  {} [label=\"{}\"];\n", id, node.value()));

    let mut next_id = id + 1;
    if let Some(children) = node.children() {
        for child in children.iter() {
            let child_id = next_id;
            next_id = dot_recursive(child, child_id, dot);
            dot.push_str(&format!("  {} -> {};\n", id, child_id));
        }
    }
    next_id
}
