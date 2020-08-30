use std::ops::{Deref, DerefMut};
use std::ptr;
use std::cmp::max;
use std::fmt;

pub type Link<T> = Option<Box<Node<T>>>;

/// a Node struct to represent a bidirectional binary tree
/// holding pointers to the parent and two children, the left and right child
pub struct Node<T: Clone> {
    elem: T,
    parent: *mut Node<T>,
    left_child: Link<T>,
    right_child: Link<T>,
}

/// implement the node
impl<T: Clone> Node<T> {
    /// create a new node.
    pub fn new(elem: T) -> Box<Self> {
        Box::new(Node {
            elem,
            parent: ptr::null_mut(),
            left_child: None,
            right_child: None,
        })
    }

    pub fn get(&self) -> &T {
        &self.elem
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.elem
    }

    /// Check if node 'is' the same as this node.
    /// This just checks that the references are for the same 'Node<T>'
    pub fn is(&self, node: &Node<T>) -> bool {
        self as *const Node<T> == node as *const Node<T>
    }

    /// return true if this node is the left child, false if not
    pub fn check_left_child(&self, node: &Node<T>) -> bool {
        match self.left_child_opt() {
            Some(child) => child.is(node),
            None => false,
        }
    }

    /// return true if this node is the right child, false if not
    pub fn check_right_child(&self, node: &Node<T>) -> bool {
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

    /// return true if this node is a leaf node, meaning it has no children.
    pub fn is_leaf(&self) -> bool {
        !self.has_left_child() && !self.has_right_child()
    }

    /// return true if this node has a valid left child and is not pointing to a null pointer
    pub fn has_left_child(&self) -> bool {
        self.left_child.is_some()
    }

    /// return true if this node has a valid right child and is not pointing to a null pointer 
    pub fn has_right_child(&self) -> bool {
        self.right_child.is_some()
    }

    /// return true if this node has a parent, false if not. 
    /// If it does not, then this node is the root of the tree.
    pub fn has_parent(&self) -> bool {
        !self.parent.is_null()
    }

    /// Safely set the left child node.
    /// Will drop any old left child node.
    pub fn set_left_child(&mut self, child: Link<T>) {
        self.take_left_child(); // Drops old left child
        if let Some(mut child) = child {
            child.set_parent(self);
            self.left_child = Some(child);
        }
    }

    /// Safely set the right child node.
    /// Will drop any old right child node.
    pub fn set_right_child(&mut self, child: Link<T>) {
        self.take_right_child(); // Drops old right child
        if let Some(mut child) = child {
            child.set_parent(self);
            self.right_child = Some(child);
        }
    }

    /// Safely set the node's parent.
    /// If the node already has a parent, this node will be removed from it.
    pub fn set_parent(&mut self, parent: *mut Node<T>) {
        self.remove_from_parent();
        self.parent = parent;
    }

    /// Returns a raw mutable reference to this node's left child.
    pub(crate) fn left_child_mut_ptr_opt(&mut self) -> Option<*mut Node<T>> {
        self.left_child.as_mut().map(|n| (&mut **n) as *mut Node<T>)
    }

    /// Returns a raw mutable reference to this node's right child.
    pub(crate) fn right_child_mut_ptr_opt(&mut self) -> Option<*mut Node<T>> {
        self.right_child.as_mut().map(|n| (&mut **n) as *mut Node<T>)
    }

    /// Safely returns a mutable reference to this node's left child.
    pub fn left_child_mut_opt(&mut self) -> Option<&mut Node<T>> {
        self.left_child.as_mut().map(|n| &mut **n)
    }

    /// Safely returns a mutable reference to this node's right child.
    pub fn right_child_mut_opt(&mut self) -> Option<&mut Node<T>> {
        self.right_child.as_mut().map(|n| &mut **n)
    }

    /// Safely returns a mutable reference to this node's parent.
    pub fn parent_mut_opt(&mut self) -> Option<&mut Node<T>> {
        if self.has_parent() {
            Some(unsafe { &mut *self.parent })
        } else {
            None
        }
    }

    /// Safely returns a reference to this node's left child.
    pub fn left_child_opt(&self) -> Option<&Node<T>> {
        self.left_child.as_ref().map(|n| &**n)
    }

    /// Safely returns a reference to this node's right child.
    pub fn right_child_opt(&self) -> Option<&Node<T>> {
        self.right_child.as_ref().map(|n| &**n)
    }

    /// Safely returns a reference to this node's parent.
    pub fn parent_opt(&self) -> Option<&Node<T>> {
        if self.has_parent() {
            Some(unsafe { &*self.parent })
        } else {
            None
        }
    }

    /// Remove and return the left child node.
    /// The returned node is owned by the caller
    pub fn take_left_child(&mut self) -> Link<T> {
        if let Some(mut child) = self.left_child.take() {
            child.parent = ptr::null_mut();
            Some(child)
        } else {
            None
        }
    }

    /// Remove and return the right child node.
    /// The returned node is owned by the caller
    pub fn take_right_child(&mut self) -> Link<T> {
        if let Some(mut child) = self.right_child.take() {
            child.parent = ptr::null_mut();
            Some(child)
        } else {
            None
        }
    }

    /// Safely remove a child node.
    fn remove_child(&mut self, child: &Node<T>) {
        let mut removed = false;
        if Some(child) == self.left_child_opt() {
            removed = true;
            self.left_child = None;
        }
        if Some(child) == self.right_child_opt() {
            assert!(!removed, "Node set as both left child and right child.");
            removed = true;
            self.right_child = None;
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

    /// return the height of this node recursively
    #[inline]    
    pub fn height(&self) -> i32 {
        1 + max(
            self.left_child_opt().map_or(0, |node| node.height()),
            self.right_child_opt().map_or(0, |node| node.height())
        )
    }

    /// return the depth of this node, meaning the number of levels down it is 
    /// from the root of the tree, recursively.
    #[inline]    
    pub fn depth(&self) -> i32 {
        match self.parent_opt() {
            Some(parent) => 1 + parent.depth(),
            None => 0
        }
    }

    /// return the size of the subtree recursively.
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
    pub fn copy(&self) -> Box<Node<T>> {
        Box::new(Node {
            elem: self.elem.clone(),
            parent: ptr::null_mut(),
            left_child: None,
            right_child: None,
        })
    }

    /// deep copy this node and it's subnodes. Recursively traverse the tree in order and
    /// thin copy the current node, then assign it's surrounding pointers recursively.
    #[inline]    
    pub fn deepcopy(&self) -> Box<Node<T>> {
        let mut temp_copy = self.copy();
        if let Some(child) = self.left_child_opt() {
            let child = child.deepcopy();
            temp_copy.set_left_child(Some(child));
        }
        if let Some(child) = self.right_child_opt() {
            let child = child.deepcopy();
            temp_copy.set_right_child(Some(child));
        }
        temp_copy
    }

    /// Randomly insert a node into the tree.
    pub fn insert_random(&mut self, node: Box<Node<T>>) {
        match rand::random() {
            true => {
                if let Some(child) = self.left_child_mut_opt() {
                    child.insert_random(node);
                } else {
                    self.set_left_child(Some(node));
                    return
                }
            },
            false => {
                if let Some(child) = self.right_child_mut_opt() {
                    child.insert_random(node);
                } else {
                    self.set_right_child(Some(node));
                    return
                }
            }
        }
    }

    /// Recursively display the node and it's subnodes
    /// Useful for visualizing the structure of the tree and debugging.
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

impl<T: Clone> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.elem
    }
}

impl<T: Clone> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elem
    }
}

impl<T: Clone> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self as *const Node<T> == other as *const Node<T>
    }
}

/// implement `Debug` for the node to give a little more information for the node and
/// make it easier to trace through a tree when a tree is displayed
impl<T: Clone> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node[{:p}]={{parent = {:?}, left = {:?}, right = {:?}}}",
          self, self.parent, self.left_child, self.right_child)
    }
}
