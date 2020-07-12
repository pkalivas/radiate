

use super::node::Node;


/// Level order iterator struct to keep track of the current position of
/// the iterator while iterating over the tree
pub struct LevelOrderIterator<'a, T: Clone> {
    pub stack: Vec<&'a Node<T>>
}



/// In order iterator for the tree. Keeps a vec to remember the current position 
/// of the tree during iteration.
pub struct InOrderIterator<'a, T: Clone> {
    pub next: Option<&'a Node<T>>,
}



/// Implement an in order iterator which allows for mutability of the 
/// nodes inside the iterator
pub struct IterMut<'a, T: Clone> {
    pub stack: Vec<Option<*mut Node<T>>>,
    phantom: std::marker::PhantomData<&'a Node<T>>,
}



impl<'a, T: Clone> LevelOrderIterator<'a, T> {
    pub fn new(root: Option<&'a Node<T>>) -> Self {
        let mut stack = Vec::new();
        if let Some(root) = root {
            stack.push(root);
        }
        Self { stack }
    }
}




/// Implement the level order iterator, all iterators in Rust call the next function
/// and because it takes a mutable reference to self, the node which is yielded by 
/// the iterator can be mutated during iteration, but will not free memory by being consumed.
impl<'a, T: Clone> Iterator for LevelOrderIterator<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_node = self.stack.pop()?;
        if let Some(child) = curr_node.right_child_opt() {
            self.stack.push(child);
        }
        if let Some(child) = curr_node.left_child_opt() {
            self.stack.push(child);
        }
        Some(curr_node)
    }
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


/// Implement the in order iterator. Will call the next function and fall down the 
/// left side of the tree till there is no left child, that is the yielded node.
/// The add the right child and continue iterating.
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




/// TODO: Try using non-stack algorithm.
impl<'a, T: Clone> IterMut<'a, T> {
    pub fn new(root: Option<&'a mut Node<T>>) -> Self {
        let mut stack = Vec::new();
        //if let Some(root) = root {
            stack.push(root.map(|n| n as *mut Node<T>));
        //}
        Self {
            stack,
            phantom: std::marker::PhantomData,
        }
    }
}

/// implement an in order iterator with lifetime 'a 
/// which allows for internal mutability of the 
/// nodes - same implementation as in_order_iter()
/// but allows for mutation
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
