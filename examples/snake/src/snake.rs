use crate::INPUT_SIZE;
use radiate::{Eval, Graph, Op};
use std::cmp::Ordering;

#[derive(Clone)]
pub enum SnakeAI<'a> {
    Graph(&'a Graph<Op<f32>>),
    NeuralNet(&'a [Vec<Vec<f32>>]),
}

impl<'a> SnakeAI<'a> {
    pub fn predict(&self, state: &[f32; INPUT_SIZE]) -> usize {
        match self {
            SnakeAI::Graph(graph) => self.predict_graph(graph, state),
            SnakeAI::NeuralNet(net) => self.predict_neural_net(net, state),
        }
    }

    fn predict_graph(&self, graph: &Graph<Op<f32>>, state: &[f32; INPUT_SIZE]) -> usize {
        // Graph.eval expects Vec<Vec<f32>>
        let input_vec = vec![state.to_vec()];
        let outputs = graph.eval(&input_vec); // Vec<Vec<f32>>
        let out = &outputs[0];

        out.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn predict_neural_net(&self, net: &[Vec<Vec<f32>>], state: &[f32; INPUT_SIZE]) -> usize {
        let activations = SnakeAI::feed_forward(net.to_vec(), state.to_vec());

        activations
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn feed_forward(layers: Vec<Vec<Vec<f32>>>, input: Vec<f32>) -> Vec<f32> {
        let mut output = input;
        for layer in layers.iter() {
            let layer_height = layer.len();
            let layer_width = layer[0].len();

            if output.len() != layer_height {
                panic!(
                    "Input size does not match layer size: {} != {}",
                    output.len(),
                    layer_width
                );
            }

            let mut new_output = Vec::new();
            for i in 0..layer_width {
                let mut sum = 0_f32;
                for j in 0..layer_height {
                    sum += layer[j][i] * output[j];
                }

                // Sigmoid activation
                new_output.push(1.0 / (1.0 + (-sum).exp()));
            }

            output = new_output;
        }

        output
    }
}
