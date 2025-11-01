use crate::TreeNode;

#[derive(Clone, Debug)]
pub struct TreePath<'a, T> {
    pub node: &'a TreeNode<T>,
    pub path: Vec<usize>,
}

#[derive(Debug)]
pub struct TreePathMut<'a, T> {
    pub node: &'a mut TreeNode<T>,
    pub path: Vec<usize>,
}

impl<T> TreeNode<T> {
    pub fn path_to(&self, index: usize) -> Option<TreePath<'_, T>> {
        let mut cur = 0;
        let mut path = Vec::new();
        let node = Self::path_to_preorder(self, index, &mut cur, &mut path)?;
        Some(TreePath { node, path })
    }

    pub fn path_to_mut(&mut self, index: usize) -> Option<TreePathMut<'_, T>> {
        let mut cur = 0;
        let mut path = Vec::new();
        let node = Self::path_to_preorder_mut(self, index, &mut cur, &mut path)?;
        Some(TreePathMut { node, path })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut TreeNode<T>> {
        let mut cur = 0;
        Self::get_mut_preorder(self, index, &mut cur)
    }

    fn get_mut_preorder<'a>(
        node: &'a mut TreeNode<T>,
        target: usize,
        cur: &mut usize,
    ) -> Option<&'a mut TreeNode<T>> {
        if *cur == target {
            return Some(node);
        }

        if let Some(children) = node.children_mut() {
            for child in children {
                *cur += 1;
                if let Some(found) = Self::get_mut_preorder(child, target, cur) {
                    return Some(found);
                }
            }
        }

        None
    }

    fn path_to_preorder<'a>(
        node: &'a TreeNode<T>,
        target: usize,
        cur: &mut usize,
        path: &mut Vec<usize>,
    ) -> Option<&'a TreeNode<T>> {
        if *cur == target {
            return Some(node);
        }

        if let Some(children) = node.children() {
            for (i, child) in children.iter().enumerate() {
                *cur += 1;
                path.push(i);
                if let Some(found) = Self::path_to_preorder(child, target, cur, path) {
                    return Some(found);
                }

                path.pop();
            }
        }

        None
    }

    fn path_to_preorder_mut<'a>(
        node: &'a mut TreeNode<T>,
        target: usize,
        cur: &mut usize,
        path: &mut Vec<usize>,
    ) -> Option<&'a mut TreeNode<T>> {
        if *cur == target {
            return Some(node);
        }

        if let Some(children) = node.children_mut() {
            for (i, child) in children.iter_mut().enumerate() {
                *cur += 1;
                path.push(i);

                if let Some(found) = Self::path_to_preorder_mut(child, target, cur, path) {
                    return Some(found);
                }

                path.pop();
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Node, Tree};

    use super::*;

    #[test]
    fn test_tree_path() {
        let tree = Tree::new(TreeNode::from((
            1,
            vec![
                TreeNode::from((2, vec![])),
                TreeNode::from((
                    3,
                    vec![TreeNode::from((4, vec![])), TreeNode::from((5, vec![]))],
                )),
            ],
        )));

        println!("{:#?}", tree);

        let path = tree.root().unwrap().path_to(3).unwrap();

        println!("{:#?}", path);
        assert_eq!(path.node.value(), &4);
        assert_eq!(path.path, vec![1, 0]);
    }
}
