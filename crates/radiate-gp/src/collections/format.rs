use super::{Tree, TreeNode};
use crate::Node;
use std::fmt::Debug;

pub trait Format {
    fn format(&self) -> String;
}

impl<T: Debug> Format for TreeNode<T> {
    fn format(&self) -> String {
        fn pretty_print_lines<T: Debug>(
            node: &TreeNode<T>,
            prefix: &str,
            is_last: bool,
            result: &mut String,
        ) {
            let connector = if is_last { "└── " } else { "├── " };
            let new_str_to_print = format!("{}{}{:?}\n", prefix, connector, node.value());
            result.push_str(&new_str_to_print);

            if let Some(children) = &node.children() {
                let len = children.len();
                for (i, child) in children.iter().enumerate() {
                    let is_last_child = i == len - 1;
                    let new_prefix = if is_last {
                        format!("{}    ", prefix)
                    } else {
                        format!("{}│   ", prefix)
                    };
                    pretty_print_lines(child, &new_prefix, is_last_child, result);
                }
            }
        }
        let mut result = String::new();
        if self.children().is_some() {
            result += "\n";
        }

        pretty_print_lines(self, "", true, &mut result);

        result
    }
}

impl<T: Debug> Format for Tree<T> {
    fn format(&self) -> String {
        self.root()
            .map(|node| node.format())
            .unwrap_or_else(|| "Empty Tree".to_string())
    }
}

impl<T: Debug> Format for Vec<Tree<T>> {
    fn format(&self) -> String {
        let mut result = String::new();
        for (i, tree) in self.iter().enumerate() {
            result += &format!("Tree {}:\n", i);
            result += &tree.format();
        }
        result
    }
}
