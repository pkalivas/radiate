use super::node::Node;

/// Level order iterator struct.
/// A stack is used to hold all nodes from the current level.
/// When the stack is empty, try pushing the node of the next level.
pub struct LevelOrderIterator<'a, T: Clone> {
    root: Option<&'a Node<T>>,
    next_level: usize,
    stack: Vec<&'a Node<T>>,
}

impl<'a, T: Clone> LevelOrderIterator<'a, T> {
    pub fn new(root: Option<&'a Node<T>>) -> Self {
        let mut iter = Self {
          root,
          next_level: 0,
          stack: Vec::new(),
        };
        // push first level
        iter.push_next_level();
        iter
    }

    /// Recurse down the stack until we reach the current level.
    /// Push the nodes at that level
    fn push_level(&mut self, root: Option<&'a Node<T>>, level: usize, curr: usize) {
        if let Some(node) = root {
            if level == curr {
                self.stack.push(node);
            } else {
                self.push_level(node.right_child_opt(), level+1, curr);
                self.push_level(node.left_child_opt(), level+1, curr);
            }
        }
    }

    /// Push all of the node at the current level.
    fn push_next_level(&mut self) {
        self.push_level(self.root, 0, self.next_level);
        self.next_level += 1;
    }
}

/// Implement the level order iterator.
impl<'a, T: Clone> Iterator for LevelOrderIterator<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // pop next node for current level.
        self.stack.pop().or_else(|| {
            // current level is finished.  Push next level.
            self.push_next_level();
            // If the stack is still empty, then we are finished.
            self.stack.pop()
        })
    }
}

/// In order iterator for the tree.
/// Since the nodes in the tree have parent references we can
/// walk up and down the tree and don't need a stack.
///
/// See details of the algorithm here: https://stackoverflow.com/questions/12850889/in-order-iterator-for-binary-tree
pub struct InOrderIterator<'a, T: Clone> {
    next: Option<&'a Node<T>>,
}

/// Find the left most node in the tree.
fn left_most<'a, T: Clone>(node: Option<&'a Node<T>>) -> Option<&'a Node<T>> {
    // The first node is the left most node in the tree.
    match node {
        Some(mut next) => {
            // find left most node from the root.
            while let Some(left) = next.left_child_opt() {
                next = left;
            }
            Some(next)
        },
        None => None,
    }
}

impl<'a, T: Clone> InOrderIterator<'a, T> {
    pub fn new(root: Option<&'a Node<T>>) -> Self {
        // The first node is the left most node in the tree.
        Self {
            next: left_most(root),
        }
    }
}

/// Implement the in order iterator.
impl<'a, T: Clone> Iterator for InOrderIterator<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Save the current next node to be returned.
        let curr_node = self.next;

        // Find the new next node.
        if let Some(mut node) = curr_node {
            // Check if we can walk right.
            if let Some(right) = node.right_child_opt() {
                // walk fully left.
                self.next = left_most(Some(right));
                return curr_node;
            }

            // walk up the tree.
            loop {
                // get parent.
                match node.parent_opt() {
                    None => {
                        // No parent.  We are back at root node, finished.
                        self.next = None;
                        return curr_node;
                    },
                    Some(parent) => {
                        // check if we are walking up from left-side
                        if parent.check_left_child(node) {
                            // The next node is the parent.
                            self.next = Some(parent);
                            return curr_node;
                        }
                        // when walking up from the right-side, keep going up.
                        node = parent;
                    },
                }
            }
        }
        curr_node
    }
}

/// Implement an in order iterator which allows for mutability of the
/// nodes inside the iterator
pub struct IterMut<'a, T: Clone> {
    stack: Vec<Option<*mut Node<T>>>,
    phantom: std::marker::PhantomData<&'a Node<T>>,
}

impl<'a, T: Clone> IterMut<'a, T> {
    pub fn new(root: Option<&'a mut Node<T>>) -> Self {
        let mut stack = Vec::new();
        // map mutable reference to raw pointer.
        stack.push(root.map(|n| n as *mut Node<T>));
        Self {
            stack,
            phantom: std::marker::PhantomData,
        }
    }
}

/// implement an in order iterator.
/// We have to use raw pointers and unsafe because the borrow checker
/// will not allow use to have more then one mutable reference to the same node.
impl<'a, T: Clone> Iterator for IterMut<'a, T> {
    type Item = &'a mut Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut curr_node) = self.stack.pop() {
            while let Some(curr) = curr_node {
                curr_node = unsafe { &mut *curr }.left_child_mut_ptr_opt();
                self.stack.push(Some(curr));
            }
        }
        self.stack.pop()?.map(|res_node| {
            let res_node = unsafe { &mut *res_node};
            self.stack.push(res_node.right_child_mut_ptr_opt());
            res_node
        })
    }
}

#[cfg(test)]
mod test {
    use crate::tree::*;

    #[test]
    fn level_order_iter() {
        //                     0
        //              ______/ \______
        //             /               \
        //            1                 2
        //        __/   \__          __/ \__
        //       /         \        /       \
        //      3           4      5         6
        //     / \         / \    / \       / \
        //    7   8       9  10  11  12   13  14
        //
        let mut nums = [7,3,8,1,9,4,10,0,11,5,12,2,13,6,14].iter().map(|n| Some(*n)).collect::<Vec<_>>();
        let tree = Tree::from_slice(&mut nums[..]);

        let root = tree.root_opt().expect("no root node");
        assert_eq!(root.get(), &0);
        let left = root.left_child_opt().expect("no left node");
        assert_eq!(left.get(), &1);
        let right = root.right_child_opt().expect("no right node");
        assert_eq!(right.get(), &2);

        for (i, n) in tree.level_order_iter().enumerate() {
            println!(" - level order[{}] = {}", i, n.get());
            assert_eq!(i, *n.get());
        }

        let mut iter = tree.level_order_iter().map(|n| n.get());
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), Some(&6));
    }

    #[test]
    fn in_order_iter() {
        let mut nums = (0..10).map(|n| Some(n)).collect::<Vec<_>>();
        let tree = Tree::from_slice(&mut nums[..]);
        println!("tree = {:?}", tree);
        for (i, n) in tree.in_order_iter().enumerate() {
            println!(" - tree[{}] = {}", i, n.get());
        }

        assert_eq!(tree.len(), 10);

        let mut iter = tree.in_order_iter().map(|n| &**n);
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
    }

    #[test]
    fn iter_mut() {
        let mut nums = (0..10).map(|n| Some(n)).collect::<Vec<_>>();
        let mut tree = Tree::from_slice(&mut nums[..]);
        println!("tree = {:?}", tree);
        for (i, n) in tree.in_order_iter().enumerate() {
            println!(" - tree[{}] = {}", i, n.get());
        }

        assert_eq!(tree.len(), 10);

        let mut iter = tree.iter_mut().map(|n| n.get_mut());
        assert_eq!(iter.next(), Some(&mut 0));
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));

        for n in tree.iter_mut() {
            *n.get_mut() *= 2;
        }

        println!("tree = {:?}", tree);
        for (i, n) in tree.in_order_iter().enumerate() {
            println!(" - tree[{}] = {}", i, n.get());
        }

        let mut iter = tree.iter_mut().map(|n| n.get_mut());
        assert_eq!(iter.next(), Some(&mut 0));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 4));
        assert_eq!(iter.next(), Some(&mut 6));
    }
}
