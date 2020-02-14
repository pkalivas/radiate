use rand::Rng;
use std::ptr;
use std::mem;
use std::cmp::max;
use std::fmt;
use std::marker::Sync;

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



    /// return true if this node is the left child of it's parent, false if not
    pub fn check_left_child(&self, node: &Node) -> bool {
        if self.has_left_child() {
           unsafe {
               return *self.left_child == *node;
           }
        }
        false
    }



    /// return true if this node is the right child of it's parent, false if not
    pub fn check_right_child(&self, node: &Node) -> bool {
        if self.has_right_child() {
            unsafe {
                return *self.right_child == *node;
            }
        }
        false
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



    /// return the height of this node recursivley
    #[inline]    
    pub fn height(&self) -> i32 {
        unsafe {
            1 + max(
                Some(&*self.left_child).map_or(0, |node| node.height()),
                Some(&*self.right_child).map_or(0, |node| node.height())
            )
        }
    }



    /// return the depth of this node, meaning the number of levels down it is 
    /// from the root of the tree, recrsive.
    #[inline]    
    pub fn depth(&self) -> i32 {
        unsafe {
            if !self.has_parent() {
                return 0;
            }
            return 1 + (*self.parent).depth()
        }
    }



    /// return the size of the subtree recrusivley. 
    #[inline]    
    pub fn size(&self) -> i32 {
        let mut result = 1;
        if !self.is_leaf() {
            if self.has_left_child() {
                unsafe { result += (*self.left_child).size(); } 
            }
            if self.has_right_child() {
                unsafe { result += (*self.right_child).size(); }
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
    pub fn deepcopy(&self) -> *mut Node {
        unsafe {
            let temp_copy = self.copy().as_mut_ptr();
            if self.has_left_child() {
                (*temp_copy).left_child = (*self.left_child).deepcopy();
                (*(*temp_copy).left_child).parent = temp_copy;
            }
            if self.has_right_child() {
                (*temp_copy).right_child = (*self.right_child).deepcopy();
                (*(*temp_copy).right_child).parent = temp_copy;
            }
            temp_copy
        }
    }



    /// Unsafe function.
    /// 
    /// Randomly insert a random node into the tree. Choose a boolean value randomly 
    /// and recurse the tree until a null_mut() pointer is found, then insert a new node.
    pub unsafe fn insert_random(&mut self, input_size: i32, output_options: &Vec<i32>) {
        match rand::random() {
            true => {
                if !self.has_left_child() {
                    self.left_child = Node::new(input_size, output_options).as_mut_ptr();
                    (*self.left_child).parent = self;
                    return
                }
                (*self.left_child).insert_random(input_size, output_options);
            },
            false => {
                if !self.has_right_child() {
                    self.right_child = Node::new(input_size, output_options).as_mut_ptr();
                    (*self.right_child).parent = self; 
                    return
                }
                (*self.right_child).insert_random(input_size, output_options);
            }
        }
    }



    /// Recrusively display the node and it's subnodes 
    /// Useful for visualizing the strucutre of the tree and debugging.
    /// Level is the depth of the tree, at the root it should be 0.
    pub fn display(&self, level: i32) {
        unsafe {
            if self.left_child != ptr::null_mut() {
                (*self.left_child).display(level + 1);
            }
            let tabs: String = (0..level)
                .map(|_| "\t")
                .collect::<Vec<_>>()
                .join("");
            println!("{}{:?}\n", tabs, self);
            if self.right_child != ptr::null_mut() {
                (*self.right_child).display(level + 1);
            }
        }
    }



}



/// This will recursivley drop all nodes in this node's 
/// subree. These are made out of raw pointers so they need to be 
/// dropped manually
impl Drop for Node {
    fn drop(&mut self) {  }
}




unsafe impl Send for Node {}

unsafe impl Sync for Node {}



/// implemented a display function for the node to display a simple representation of the node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let address: u64 = unsafe { mem::transmute(self) };
        write!(f, "Node=[{}]", self.output)
    }
}



/// implement debut for the node to give a little more information for the node and 
/// make it easier to trace through a tree when a tree is displayed
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            // let address: u64 = mem::transmute(self);
            // let left: u64 = if self.has_left_child() { mem::transmute(&*self.left_child) } else { 0x64 };
            // let right: u64 = if self.has_right_child() { mem::transmute(&*self.right_child) } else { 0x64 };
            write!(f, "Node=[{}]", self.output)
        }
    }
}

