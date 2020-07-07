use rand::Rng;
use std::ptr;
use std::cmp::max;
use std::fmt;

use super::network::NeuralNetwork;




/// a Node struct to represent a bidirectional binary tree
/// holding pointers to the parent and two children, the left and right child
/// The node also holds an input size which is the expected size of the input vector 
/// for the neural network held within the node. The output represetns the postion of the 
/// node, meaning that if it is a leaf, it will return the output from a get output function
#[derive(PartialEq)]
pub struct Node {
    pub parent: *mut Node,
    pub left_child: *mut Node,
    pub right_child: *mut Node,
    pub neural_network: NeuralNetwork,
    pub input_size: i32,
    pub output: u8
}




/// implement the node
impl Node {

    /// create a new node with a given input size and a list of possible output options 
    /// 
    /// From the list of output_options the node will choose an output,
    /// from the input_size the node will create a randomly generated 
    /// neural network.
    pub fn new(input_size: i32, output_options: &Vec<i32>) -> Self {
        let mut r = rand::thread_rng();
        let output = output_options[r.gen_range(0, output_options.len())] as u8;
        Node {
            parent: ptr::null_mut(),
            left_child: ptr::null_mut(),
            right_child: ptr::null_mut(),
            neural_network: NeuralNetwork::new(input_size).fill_random(),
            input_size,
            output
        }
    }



    /// return a node as a raw mutable pointer to a node
    /// this is a safe function however dereferencing the output is not
    /// also creating a box and putting it into raw includes a small amount of overhead 
    pub fn as_mut_ptr(self) -> *mut Node {
        Box::into_raw(Box::new(self))
    }


    /// Check if node 'is' the same as this node.
    /// This just checks that the references are for the same 'Node'
    pub fn is(&self, node: &Node) -> bool {
        self as *const Node == node as *const Node
    }

    /// return true if this node is the left child, false if not
    pub fn check_left_child(&self, node: &Node) -> bool {
        match self.left_child_opt() {
            Some(child) => child.is(node),
            None => false,
        }
    }

    /// return true if this node is the right child, false if not
    pub fn check_right_child(&self, node: &Node) -> bool {
        match self.right_child_opt() {
            Some(child) => child.is(node),
            None => false,
        }
    }


    /// return true if this node is the left child of it's parent, false if not
    /// returns `None` if node doesn't have a parent.
    pub fn is_left_child(&self) -> Option<bool> {
        match self.parent_opt() {
            Some(parent) => Some(parent.check_left_child(self)),
            None => None,
        }
    }


    /// return true if this node has no children, meaning it has no children.
    /// A node that is a leaf is the only node which can return an output.
    pub fn is_leaf(&self) -> bool {
        !self.has_left_child() && !self.has_right_child()
    }



    /// return true if this node has a valid left child and is not pointing to a null pointer
    pub fn has_left_child(&self) -> bool {
        self.left_child != ptr::null_mut()
    }



    /// return true if this node has a valid right child and is not pointing to a null pointer 
    pub fn has_right_child(&self) -> bool {
        self.right_child != ptr::null_mut()
    }



    /// return true if this node has a parent, false if not. 
    /// If it does not, then this node is the root of the tree.
    pub fn has_parent(&self) -> bool {
        self.parent != ptr::null_mut()
    }


    /// Safely set the left child node.
    /// Will drop any old left child node.
    pub fn set_left_child(&mut self, child: *mut Node) {
        self.take_left_child(); // Drops old left child
        if child != ptr::null_mut() {
            self.left_child = child;
            unsafe {
                (*child).set_parent(self);
            }
        }
    }

    /// Safely set the right child node.
    /// Will drop any old right child node.
    pub fn set_right_child(&mut self, child: *mut Node) {
        self.take_right_child(); // Drops old right child
        if child != ptr::null_mut() {
            self.right_child = child;
            unsafe {
                (*child).set_parent(self);
            }
        }
    }

    /// Safely set the node's parent.
    /// If the node already has a parent, this node will be removed from it.
    pub fn set_parent(&mut self, parent: *mut Node) {
        self.remove_from_parent();
        self.parent = parent;
    }

    /// Safely returns a mutable reference to this node's left child.
    pub fn left_child_mut_opt(&self) -> Option<&mut Node> {
        if self.has_left_child() {
            Some(unsafe { &mut *self.left_child })
        } else {
            None
        }
    }

    /// Safely returns a mutable reference to this node's right child.
    pub fn right_child_mut_opt(&self) -> Option<&mut Node> {
        if self.has_right_child() {
            Some(unsafe { &mut *self.right_child })
        } else {
            None
        }
    }

    /// Safely returns a mutable reference to this node's parent.
    pub fn parent_mut_opt(&self) -> Option<&mut Node> {
        if self.has_parent() {
            Some(unsafe { &mut *self.parent })
        } else {
            None
        }
    }

    /// Safely returns a reference to this node's left child.
    pub fn left_child_opt(&self) -> Option<&Node> {
        if self.has_left_child() {
            Some(unsafe { &*self.left_child })
        } else {
            None
        }
    }

    /// Safely returns a reference to this node's right child.
    pub fn right_child_opt(&self) -> Option<&Node> {
        if self.has_right_child() {
            Some(unsafe { &*self.right_child })
        } else {
            None
        }
    }

    /// Safely returns a reference to this node's parent.
    pub fn parent_opt(&self) -> Option<&Node> {
        if self.has_parent() {
            Some(unsafe { &*self.parent })
        } else {
            None
        }
    }


    /// Remove and return the left child node.
    /// The returned node is owned by the caller
    pub fn take_left_child(&mut self) -> Option<Box<Node>> {
        if self.has_left_child() {
            let child = unsafe { Box::from_raw(self.left_child) };
            self.left_child = ptr::null_mut();
            Some(child)
        } else {
            None
        }
    }

    /// Remove and return the right child node.
    /// The returned node is owned by the caller
    pub fn take_right_child(&mut self) -> Option<Box<Node>> {
        if self.has_right_child() {
            let child = unsafe { Box::from_raw(self.right_child) };
            self.right_child = ptr::null_mut();
            Some(child)
        } else {
            None
        }
    }

    /// Safely remove a child node.
    fn remove_child(&mut self, child: *mut Node) {
        let mut removed = false;
        if child == self.left_child {
            removed = true;
            self.left_child = ptr::null_mut();
        }
        if child == self.right_child {
            assert!(!removed, "Node set as both left child and right child.");
            removed = true;
            self.right_child = ptr::null_mut();
        }
        assert!(removed, "Node isn't a child of this node.");
    }

    /// Safely detach this node from it's parent.
    pub fn remove_from_parent(&mut self) {
        if self.has_parent() {
            let parent = unsafe { &mut *self.parent };
            self.parent = ptr::null_mut();
            parent.remove_child(self);
        }
    }

    /// return the height of this node recursivley
    #[inline]    
    pub fn height(&self) -> i32 {
        1 + max(
            self.left_child_opt().map_or(0, |node| node.height()),
            self.right_child_opt().map_or(0, |node| node.height())
        )
    }



    /// return the depth of this node, meaning the number of levels down it is 
    /// from the root of the tree, recrsive.
    #[inline]    
    pub fn depth(&self) -> i32 {
        match self.parent_opt() {
            Some(parent) => 1 + parent.depth(),
            None => 0
        }
    }



    /// return the size of the subtree recrusivley. 
    #[inline]    
    pub fn size(&self) -> i32 {
        let mut result = 1;
        if !self.is_leaf() {
            if let Some(left_child) = self.left_child_opt() {
                result += left_child.size();
            }
            if let Some(right_child) = self.right_child_opt() {
                result += right_child.size();
            }
        }
        result
    }


   
    /// Return a thin copy of this node, meaning keep all information besides the family pointers,
    /// these are nulled-out in order to avoid dangling or circular references.
    #[inline]
    pub fn copy(&self) -> Node {
        Node {
            parent: ptr::null_mut(),
            left_child: ptr::null_mut(),
            right_child: ptr::null_mut(),
            neural_network: self.neural_network.clone(),
            input_size: self.input_size,
            output: self.output
        }
    }



    /// deep copy this node and it's subnodes. Recursivley traverse the tree in order and 
    /// thin copy the current node, then assign it's surroudning pointers recrusivley.
    #[inline]    
    pub fn deepcopy(&self) -> Box<Node> {
        let mut temp_copy = Box::new(self.copy());
        if let Some(child) = self.left_child_opt() {
            let child = Box::into_raw(child.deepcopy());
            temp_copy.set_left_child(child);
        }
        if let Some(child) = self.right_child_opt() {
            let child = Box::into_raw(child.deepcopy());
            temp_copy.set_right_child(child);
        }
        temp_copy
    }



    /// Randomly insert a random node into the tree. Choose a boolean value randomly 
    /// and recurse the tree until a null_mut() pointer is found, then insert a new node.
    pub fn insert_random(&mut self, input_size: i32, output_options: &Vec<i32>) {
        match rand::random() {
            true => {
                if let Some(child) = self.left_child_mut_opt() {
                    child.insert_random(input_size, output_options);
                } else {
                    self.set_left_child(Node::new(input_size, output_options).as_mut_ptr());
                    return
                }
            },
            false => {
                if let Some(child) = self.right_child_mut_opt() {
                    child.insert_random(input_size, output_options);
                } else {
                    self.set_right_child(Node::new(input_size, output_options).as_mut_ptr());
                    return
                }
            }
        }
    }



    /// Recrusively display the node and it's subnodes 
    /// Useful for visualizing the strucutre of the tree and debugging.
    /// Level is the depth of the tree, at the root it should be 0.
    pub fn display(&self, level: i32) {
        if let Some(child) = self.left_child_opt() {
            child.display(level + 1);
        }
        let tabs: String = (0..level)
            .map(|_| "\t")
            .collect::<Vec<_>>()
            .join("");
        println!("{}{:?}\n", tabs, self);
        if let Some(child) = self.right_child_opt() {
            child.display(level + 1);
        }
    }



}



/// This will recursivley drop all nodes in this node's 
/// subree. These are made out of raw pointers so they need to be 
/// dropped manually
impl Drop for Node {
    fn drop(&mut self) {  }
}




/// implemented a display function for the node to display a simple representation of the node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node=[{}]", self.output)
    }
}



/// implement debut for the node to give a little more information for the node and 
/// make it easier to trace through a tree when a tree is displayed
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node[{:p}]={{parent = {:?}, left = {:?}, right = {:?}, output = {}}}",
          self, self.parent, self.left_child, self.right_child, self.output)
    }
}
