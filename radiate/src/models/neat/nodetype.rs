
use std::collections::HashMap;
use super::activation::Activation;

/// Define a type of node to create - default is Dense which
/// is a normal feed forward neuron
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeType {
    Dense,
    LSTM,
    Recurrent
}


//////////////////////////////
/// I Don't like the current design of this...it feels lazy. There has got to be a more idomatic way to introduce mutliple nodes
/// to the graph. I'm struggling to move past traditional oop in this sense.
/// 
/// Expect this to be changed or tweaked sometime within the next week-10 days. 
/// Shouldn't effect any current use cases.
//////////////////////////////


/// implement the nodetype enum
impl NodeType {


    /// activate the node 
    #[inline]
    pub fn activate(&self, incoming: &HashMap<i32, Option<f64>>, activation_function: &Activation, prev_value: &Option<f64>, cell_state: &Option<f64>) -> (Option<f64>, Option<f64>) {
        match self {
            Self::Dense => self.dense_output(incoming, activation_function),
            Self::LSTM => self.lstm_output(incoming, prev_value, cell_state),
            Self::Recurrent => self.recurrent_output()
        }
    }



    /// calculate the output of a normal dense node
    #[inline]
    fn dense_output(&self, incoming: &HashMap<i32, Option<f64>>, activation_function: &Activation) -> (Option<f64>, Option<f64>) {
        let mut total = 0.0;
        for value in incoming.values() {
            match value {
                Some(v) => total += v,
                None => panic!("failed to activate.")
            }
        }
        (None, Some(activation_function.activate(total)))
    }



    /// calculate the output of a long-short term memory node
    #[inline]
    fn lstm_output(&self, incoming: &HashMap<i32, Option<f64>>, prev_value: &Option<f64>, cell_state: &Option<f64>) -> (Option<f64>, Option<f64>) {
        let mut total = 0.0;
        for value in incoming.values() {
            match value {
                Some(v) => total += v,
                None => panic!("failed to activate.")
            }
        }
        total += if prev_value.is_none() { 0.0 } else { prev_value.unwrap() };
        // calculate the forget and input gate
        // update the cell state
        // calculate the output gate
        let temp_cell_state = if cell_state.is_none() { 0.0 } else { cell_state.unwrap() };
        let forget_gate = Activation::Sigmoid.activate(total);
        let input_gate = forget_gate * Activation::Tahn.activate(total);
        let new_cell_state = (temp_cell_state * forget_gate) + input_gate;
        let output_gate = forget_gate * Activation::Tahn.activate(new_cell_state);
        (Some(new_cell_state), Some(output_gate))
    }



    #[inline]
    fn recurrent_output(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }


}