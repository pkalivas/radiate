use crate::{Direction, Graph, Node, NodeType, Tree, TreeNode};
use std::fmt::{Debug, Display};

pub trait ToDot {
    fn to_dot(&self) -> String;
}

impl<T> ToDot for Graph<T>
where
    T: Debug,
{
    fn to_dot(&self) -> String {
        let mut dot = String::new();

        dot += "digraph G {\n";

        for (i, node) in self.iter().enumerate() {
            let node_label = match node.node_type() {
                NodeType::Input => format!(
                    "{} [label=\"{:?}\", shape=box, color=blue];",
                    i,
                    node.value()
                ),
                NodeType::Output => {
                    format!(
                        "{} [label=\"{:?}\", shape=box, color=green];",
                        i,
                        node.value()
                    )
                }
                NodeType::Vertex => {
                    format!(
                        "{} [label=\"{:?}\", shape=circle, color=orange];",
                        i,
                        node.value()
                    )
                }
                NodeType::Edge => {
                    format!(
                        "{} [label=\"{:?}\", shape=diamond, color=gray];",
                        i,
                        node.value()
                    )
                }
                _ => continue,
            };
            dot += &node_label;
            dot += "\n";
        }

        for (i, node) in self.iter().enumerate() {
            for incoming in node.incoming() {
                let edge_style = if self[*incoming].direction() == Direction::Forward {
                    " [style=solid]"
                } else {
                    " [style=dashed]"
                };

                dot += &format!("  {} -> {}{};\n", incoming, i, edge_style);
            }
        }

        dot += "}\n";

        dot
    }
}

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
