use crate::expr;
use crate::expr::Expr;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct NodeFactory<T: Clone> {
    inputs: Vec<Expr<T>>,
    operations: Vec<Expr<T>>,
    outputs: Vec<Expr<T>>,
}

impl<T> NodeFactory<T>
where
    T: Clone + Default,
{
    pub fn new() -> Self {
        NodeFactory {
            inputs: Vec::new(),
            operations: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn get_inputs(&self) -> &Vec<Expr<T>> {
        &self.inputs
    }

    pub fn get_operations(&self) -> &Vec<Expr<T>> {
        &self.operations
    }

    pub fn get_outputs(&self) -> &Vec<Expr<T>> {
        &self.outputs
    }

    pub fn inputs(&mut self, values: Vec<Expr<T>>) {
        self.inputs = values.clone();
    }

    pub fn outputs(&mut self, values: Vec<Expr<T>>) {
        self.outputs = values.clone();
    }

    pub fn gates(&mut self, values: Vec<Expr<T>>) {
        self.operations = values.clone();
    }

    // pub fn new_node(&self, index: usize, node_type: NodeType) -> Node<T>
    // where
    //     T: Default,
    // {
    //     if let Some(values) = self.node_values.get(&node_type) {
    //         return match node_type {
    //             NodeType::Input => {
    //                 let value = values[index % values.len()].clone();
    //                 Node::new(index, node_type, value)
    //             }
    //             _ => {
    //                 let value = random_provider::choose(values);
    //                 Node::new(index, node_type, value.new_instance())
    //             }
    //         };
    //     }
    //
    //     Node::new(index, node_type, Expr::default())
    // }

    pub fn regression(input_size: usize) -> NodeFactory<f32> {
        let inputs = (0..input_size).map(expr::var).collect::<Vec<Expr<f32>>>();
        let mut factory = NodeFactory::new();
        factory.inputs(inputs.clone());
        factory.gates(vec![
            // expr::add(),
            // expr::sub(),
            // expr::mul(),
            // expr::div(),
            // expr::pow(),
            // expr::sqrt(),
            // expr::exp(),
            // expr::abs(),
            // expr::log(),
            // expr::sin(),
            // expr::cos(),
            // expr::tan(),
            // expr::sum(),
            // expr::prod(),
            // expr::max(),
            // expr::min(),
            // expr::ceil(),
            // expr::floor(),
            // expr::gt(),
            // expr::lt(),
            expr::sigmoid(),
            expr::tanh(),
            expr::relu(),
            expr::linear(),
            expr::sum(),
            expr::prod(),
            expr::max(),
            expr::min(),
            expr::mish(),
            expr::leaky_relu(),
            expr::softplus(),
            expr::sum(),
            expr::prod(),
        ]);
        factory.outputs(vec![expr::linear()]);
        factory
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new() {
        // let factory: NodeFactory<Expr<f32>> = NodeFactory::new();
        // assert!(factory.node_values.is_empty());
    }
}
