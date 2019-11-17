

use super::node::Node;


/// Level order iterator struct to keep track of the current position of
/// the iterator while iterating over the tree
pub struct LevelOrderIterator<'a> {
    pub stack: Vec<&'a Node>
}



/// In order iterator for the tree. Keeps a vec to remember the current position 
/// of the tree during iteration.
pub struct InOrderIterator<'a> {
    pub stack: Vec<&'a Node>,
}



/// Implement an in order iterator which allows for mutability of the 
/// nodes inside the iterator
pub struct IterMut<'a> {
    pub stack: Vec<&'a mut Node>,
}





/// Implement the level order iterator, all iterators in Rust call the next function
/// and because it takes a mutable reference to self, the node which is yielded by 
/// the iterator can be mutated during iteration, but will not free memory by being consumed.
impl<'a> Iterator for LevelOrderIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_node = self.stack.pop()?;
        unsafe {
            if curr_node.has_right_child() {
                self.stack.push(&*curr_node.right_child); 
            }
            if curr_node.has_left_child() {
                self.stack.push(&*curr_node.left_child);
            }
        }
        Some(curr_node)    
    }
}




/// Implement the in order iterator. Will call the next function and fall down the 
/// left side of the tree till there is no left child, that is the yielded node.
/// The add the right child and continue iterating.
impl<'a> Iterator for InOrderIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let mut curr_node = self.stack.pop();
        unsafe {
            while let Some(curr) = curr_node {
                curr_node = Some(&*curr.left_child);
                self.stack.push(&*curr);
            }
            let res_node = self.stack.pop()?;
            self.stack.push(&*res_node.right_child);
            Some(res_node)
        }
    }
}




/// implement an in order iterator with lifetime 'a 
/// which allows for internal mutability of the 
/// nodes - same implementation as in_order_iter()
/// but allows for mutation
impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Node;

    fn next(&mut self) -> Option<Self::Item> {
        let mut curr_node = self.stack.pop();
        unsafe {
            while let Some(curr) = curr_node {
                curr_node = Some(&mut *curr.left_child);
                self.stack.push(&mut *curr);
            }
            let res_node = self.stack.pop()?;
            self.stack.push(&mut *res_node.right_child);
            Some(res_node)
        }
    }
}


