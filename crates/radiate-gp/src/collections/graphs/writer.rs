use crate::{Direction, Graph, Node, NodeType, ToDot};
use std::fmt::Display;

impl<T> ToDot for Graph<T>
where
    T: Display,
{
    fn to_dot(&self) -> String {
        let mut dot = String::new();

        dot += "digraph G {\n";

        for (i, node) in self.iter().enumerate() {
            let node_label = match node.node_type() {
                NodeType::Input => format!(
                    "{} [label=\"I {}\", shape=box, color=blue];",
                    i,
                    node.value()
                ),
                NodeType::Output => {
                    format!(
                        "{} [label=\"O {}\", shape=box, color=green];",
                        i,
                        node.value()
                    )
                }
                NodeType::Vertex => {
                    format!(
                        "{} [label=\"V {}\", shape=circle, color=orange];",
                        i,
                        node.value()
                    )
                }
                NodeType::Edge => {
                    format!(
                        "{} [label=\"E {}\", shape=diamond, color=gray];",
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
