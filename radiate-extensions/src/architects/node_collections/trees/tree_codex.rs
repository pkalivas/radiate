use crate::NodeFactory;



pub struct TreeCodex<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub depth: usize,
    pub factory: &'a NodeFactory<T>
}