extern crate rand;

use std::fmt;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rand::Rng;
use rand::seq::SliceRandom;

use uuid::Uuid;

use super::{
    layertype::LayerType,
    layer::Layer,
    vectorops
};
use super::super::{
    id::*,
    neuron::*,
    edge::*,
    tracer::Tracer,
    neatenv::NeatEnvironment,
    neurontype::NeuronType,
    activation::Activation,
    direction::NeuronDirection
};

use crate::Genome;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dense {
    pub inputs: Vec<NeuronId>,
    pub outputs: Vec<NeuronId>,
    pub nodes: Vec<Neuron>,
    pub edges: Vec<Edge>,
    pub edge_innov_map: HashMap<Uuid, EdgeId>,
    pub trace_states: Option<Tracer>,
    pub layer_type: LayerType,
    pub activation: Activation
}

impl Dense {
    /// create a new fully connected dense layer.
    /// Each input is connected to each output with a randomly generated weight attached to the connection
    pub fn new(num_in: u32, num_out: u32, layer_type: LayerType, activation: Activation) -> Self {
        let mut layer = Dense {
            inputs: vec![],
            outputs: vec![],
            nodes: vec![],
            edges: vec![],
            edge_innov_map: HashMap::new(),
            trace_states: None, 
            layer_type,
            activation
        };

        let mut inputs = Vec::with_capacity(num_in as usize);
        for _ in 0..num_in as usize {
            let node_id = layer.make_node(NeuronType::Input, activation, NeuronDirection::Forward);
            inputs.push(node_id);
        }
        let mut outputs = Vec::with_capacity(num_out as usize);
        for _ in 0..num_out as usize {
            let node_id = layer.make_node(NeuronType::Output, activation, NeuronDirection::Forward);
            outputs.push(node_id);
        }

        let mut r = rand::thread_rng();
        for node_in in inputs.iter() {
            for node_out in outputs.iter() {
                let weight = r.gen::<f32>() * 2.0 - 1.0;
                layer.make_edge(*node_in, *node_out, weight);
            }
        }
        layer.inputs = inputs;
        layer.outputs = outputs;

        layer
    }

    /// Make a new node
    fn make_node(&mut self, neuron_type: NeuronType, activation: Activation, direction: NeuronDirection) -> NeuronId {
        let node_id = NeuronId::new(self.nodes.len());
        let node = Neuron::new(node_id, neuron_type, activation, direction);
        // Create a new node and add it to the node list.
        self.nodes.push(node);

        node_id
    }

    /// Make a new edge
    fn make_edge(&mut self, src: NeuronId, dst: NeuronId, weight: f32) -> EdgeId {
        let edge_id = EdgeId::new(self.edges.len());
        // Create a new edge and add it to the edge list.
        let edge = Edge::new(edge_id, src, dst, weight, true);
        edge.link_nodes(&mut self.nodes);

        self.edge_innov_map.insert(edge.innov, edge_id);
        self.edges.push(edge);

        edge_id
    }

    /// Disable an edge and unlink the nodes.
    fn disable_edge(&mut self, edge_id: EdgeId) {
        let edges = &mut self.edges;
        // disable edge and unlink the nodes
        if let Some(edge) = edges.get_mut(edge_id.index()) {
          edge.disable(&mut self.nodes)
        }
    }

    /// Get edge by innov
    pub fn get_edge_by_innov(&self, innov: &Uuid) -> Option<&Edge> {
        self.edge_innov_map.get(innov).and_then(|edge_id| self.edges.get(edge_id.index()))
    }

    /// Check if this layer contains an edge.
    pub fn contains_edge(&self, innov: &Uuid) -> bool {
        self.get_edge_by_innov(innov).is_some()
    }

    /// reset all the neurons in the network so they can be fed forward again
    fn reset_neurons(&mut self) {
        for val in self.nodes.iter_mut() {
            val.reset_neuron();
        }
    }

    /// get the outputs from the layer in a vec form
    pub fn get_outputs(&self) -> Option<Vec<f32>> {
        let result = self.outputs
            .iter()
            .map(|x| {
                self.nodes.get(x.index()).unwrap().activated_value
            })
            .collect::<Vec<_>>();
        Some(result)
    }

    /// Add a node to the network by getting a random edge 
    /// and inserting the new node inbetween that edge's source
    /// and destination nodes. The old weight is pushed forward 
    /// while the new weight is randomly chosen and put between the 
    /// old source node and the new node
    pub fn add_node(&mut self, activation: Activation, direction: NeuronDirection) {
        // create a new node to insert inbetween the sending and receiving nodes 
        let new_node_id = self.make_node(NeuronType::Hidden, activation, direction);

        // get a random edge to insert the node into
        let curr_edge = self.random_edge().clone();

        // create two new edges that connect the src and the new node and the 
        // new node and dst, then disable the current edge 
        self.make_edge(curr_edge.src, new_node_id, 1.0);
        self.make_edge(new_node_id, curr_edge.dst, curr_edge.weight);

        // disable current edge
        self.disable_edge(curr_edge.id);
    }

    /// add a connection to the network. Randomly get a sending node that cannot 
    /// be an output and a receiving node which is not an input node, the validate
    /// that the desired connection can be made. If it can be, make the connection
    /// with a weight of .5 in order to minimally impact the network 
    pub fn add_edge(&mut self) {
        // get a valid sending neuron
        let sending = self.random_node_not_of_type(NeuronType::Output);
        // get a vaild receiving neuron
        let receiving = self.random_node_not_of_type(NeuronType::Input);

        // determine if the connection to be made is valid 
        if self.valid_connection(sending, receiving) {
            // if the connection is valid, make it and wire the nodes to each
            let mut r = rand::thread_rng();
            self.make_edge(sending, receiving, r.gen::<f32>());
        }
    }

    /// Test whether the desired connection is valid or not by testing to see if 
    /// 1.) it is recursive
    /// 2.) the connection already exists
    /// 3.) the desired connection would create a cycle in the graph
    /// if these are all false, then the connection can be made
    fn valid_connection(&self, sending: NeuronId, receiving: NeuronId) -> bool {
        if sending == receiving {
            return false
        } else if self.exists(sending, receiving) {
            return false
        } else if self.cyclical(sending, receiving) {
            return false
        }
        true
    }

    /// check to see if the connection to be made would create a cycle in the graph
    /// and therefore make it network invalid and unable to feed forward
    fn cyclical(&self, sending: NeuronId, receiving: NeuronId) -> bool {
        let recv_node = self.nodes.get(receiving.index()).unwrap();
        // dfs stack which gets the receiving Neuron<dyn neurons> outgoing connections
        let mut stack = recv_node.outgoing_edges()
            .iter()
            .map(|x| self.edges.get(x.index()).unwrap().dst.index())
            .collect::<Vec<_>>();

        // while the stack still has nodes, continue
        while let Some(node_idx) = stack.pop() {
            // if the current node is the same as the sending, this would cause a cycle
            // else add all the current node's outputs to the stack to search through
            let curr = self.nodes.get(node_idx).unwrap();
            if curr.id == sending {
                return true;
            }
            for i in curr.outgoing_edges().iter() {
                let edge = self.edges.get(i.index()).unwrap();
                stack.push(edge.dst.index());
            }
        }
        false
    }

    /// check if the desired connection already exists within he network, if it does then
    /// we should not be creating the connection.
    fn exists(&self, sending: NeuronId, receiving: NeuronId) -> bool {
        for val in self.edges.iter() {
            if val.src == sending && val.dst == receiving {
                return true
            }
        }
        false
    }

    /// get a random node from the network
    fn random_node(&self) -> &Neuron {
        let index = rand::thread_rng().gen_range(0, self.nodes.len());
        let node = self.nodes.get(index)
            .expect("Failed to get random node");
        return node;
    }

    /// get a random node from the network not of the specific type
    fn random_node_not_of_type(&self, node_type: NeuronType) -> NeuronId {
        loop {
            let node = self.random_node();
            if node.neuron_type != node_type {
                break node.id;
            }
        }
    }


    /// get a random connection from the network
    fn random_edge(&self) -> &Edge {
        let index = rand::thread_rng().gen_range(0, self.edges.len());
        self.edges.get(index)
            .expect("Failed to get random edge")
    }



    /// give input data to the input nodes in the network and return a vec
    /// that holds the innovation numbers of the input nodes for a dfs traversal 
    /// to feed forward those inputs through the network
    fn give_inputs(&mut self, data: &Vec<f32>) -> Vec<NeuronId> {
        assert!(data.len() == self.inputs.len());
        let mut ids = Vec::with_capacity(self.inputs.len());
        for (node_id, input) in self.inputs.iter().zip(data.iter()) {
            let node = self.nodes.get_mut(node_id.index()).unwrap();
            node.activated_value = *input;
            ids.push(*node_id);
        }
        ids
    }



    /// Edit the weights in the network randomly by either uniformly perturbing
    /// them, or giving them an entire new weight all together
    fn edit_weights(&mut self, editable: f32, size: f32) {
        let mut r = rand::thread_rng();
        for edge in self.edges.iter_mut() {
            if r.gen::<f32>() < editable {
                edge.weight = r.gen::<f32>();
            } else {
                edge.weight *= r.gen_range(-size, size);
            }
        }
        for node in self.nodes.iter_mut() {
            if r.gen::<f32>() < editable {
                node.bias = r.gen::<f32>();
            } else {
                node.bias *= r.gen_range(-size, size);
            }
        }
    }



    /// get the states of the output neurons. This allows softmax and other specific actions to 
    /// be taken where knowledge of more than just the immediate neuron's state must be known
    pub fn get_output_states(&self) -> Vec<f32> {
        self.outputs
            .iter()
            .map(|x| {
                let output_neuron = self.nodes.get(x.index()).unwrap();
                output_neuron.current_state
            })
            .collect::<Vec<_>>()
    }



    /// Because the output neurons might need to be seen togehter, this must be called to 
    /// set their values before finishing the feed forward function
    pub fn set_output_values(&mut self) {
        let vals = self.get_output_states();
        let (act, d_act) = match self.activation {
            Activation::Softmax => {
                let act = vectorops::softmax(&vals);
                let d_act = vectorops::d_softmax(&act);
                (act, d_act)
            },
            _ => {
                let act = vectorops::element_activate(&vals, self.activation);
                let d_act = vectorops::element_deactivate(&vals, self.activation);
                (act, d_act)
            }
        };
        for (i, neuron_id) in self.outputs.iter().enumerate() {
            let curr_neuron = self.nodes.get_mut(neuron_id.index()).unwrap();
            curr_neuron.activated_value = act[i];
            curr_neuron.deactivated_value = d_act[i];
        }
    }



    /// take a snapshot of the neuron's values at this time step if trace is enabled
    pub fn update_traces(&mut self) {
        if let Some(tracer) = &mut self.trace_states {
            for node in self.nodes.iter() {
                tracer.update_neuron_activation(&node.id, node.activated_value);
                tracer.update_neuron_derivative(&node.id, node.deactivated_value);
            }
            tracer.index += 1;
        }
    }
}


#[typetag::serde]
impl Layer for Dense {
    /// Feed a vec of inputs through the network, will panic! if 
    /// the shapes of the values do not match or if something goes 
    /// wrong within the feed forward process.
    fn forward(&mut self, data: &Vec<f32>) -> Option<Vec<f32>> {
        // reset the network by clearing the previous outputs from the neurons 
        // this could be done more efficently if i didn't want to implement backprop
        // or recurent nodes, however this must be done this way in order to allow for the 
        // needed values for those algorithms to remain while they are needed 
        // give the input data to the input neurons and return back 
        // a stack to do a graph traversal to feed the inputs through the network
        self.reset_neurons();
        let mut path = self.give_inputs(data);

        // node_updates is used to split immutable & mutable code.
        let mut node_updates = Vec::with_capacity(self.inputs.len());

        // while the path is still full, continue feeding forward 
        // the data in the network, this is basically a dfs traversal
        while let Some(node_id) = path.pop() {

            // remove the top elemet to propagate it's value
            let curr_node = self.nodes.get(node_id.index())?;

            // no node should be in the path if it's value has not been set 
            // iterate through the current nodes outgoing connections 
            // to get its value and give that value to it's connected node
            for edge_innov in curr_node.outgoing_edges().iter() {

                // if the currnet edge is active in the network, we can propagate through it
                let curr_edge = self.edges.get(edge_innov.index())?;
                if curr_edge.active {
                    let activated_value = curr_edge.calculate(curr_node.activated_value);
                    node_updates.push((curr_edge.dst, curr_edge.id, activated_value));
                }
            }

            // apply pending node updates.
            for &(dst, edge, value) in node_updates.iter() {
                let receiving_node = self.nodes.get_mut(dst.index())?;
                receiving_node.set_incoming(edge, Some(value));

                // if the node can be activated, activate it and store it's value
                // only activated nodes can be added to the path, so if it's activated
                // add it to the path so the values can be propagated through the network
                if receiving_node.is_ready() {
                    receiving_node.activate();
                    path.push(receiving_node.id);
                }
            }
            // clear node updates.
            node_updates.clear();
        }

        // once we've made it through the network, the outputs should all
        // have calculated their values. Gather the values and return the vec
        self.set_output_values();
        self.update_traces();
        self.get_outputs()
    }


    /// Backpropagation algorithm, transfer the error through the network and change the weights of the
    /// edges accordinly, this is pretty straight forward due to the design of the neat graph
    fn backward(&mut self, error: &Vec<f32>, learning_rate: f32) -> Option<Vec<f32>> {
        // feed forward the input data to get the output in order to compute the error of the network
        // create a dfs stack to step backwards through the network and compute the error of each neuron
        // then insert that error in a hashmap to keep track of innov of the neuron and it's error 
        let mut path = Vec::with_capacity(self.inputs.len());
        for (index, id) in self.outputs.iter().enumerate() {
            let node = self.nodes.get_mut(id.index()).unwrap();
            node.error = error[index];
            path.push(*id);
        }

        // edge_updates is used to split immutable & mutable node access.
        let mut edge_updates = Vec::with_capacity(self.inputs.len());

        // step through the network backwards and adjust the weights
        while path.len() > 0 {
            // get the current node and it's error 
            let curr_node = self.nodes.get_mut(path.pop()?.index())?;
            let curr_error = curr_node.error;
            let step = match &self.trace_states {
                Some(tracer) => curr_error * tracer.neuron_derivative(curr_node.id),
                None => curr_error * curr_node.deactivated_value
            } * learning_rate;

            // reset the nodes error if it isnt an input node 
            if curr_node.neuron_type != NeuronType::Input {
                curr_node.bias += learning_rate * curr_error;
                curr_node.error = 0.0;
            }

            // iterate through each of the incoming edes to this neuron and adjust it's weight
            // and add it's error to the errros map
            for incoming_edge_id in curr_node.incoming_edges().iter() {
                edge_updates.push(*incoming_edge_id);
            }

            // apply pending edge updates.
            for incoming_edge_id in edge_updates.iter() {
                let curr_edge = self.edges.get_mut(incoming_edge_id.index())?;

                // if the current edge is active, then it is contributing to the error and we need to adjust it
                if curr_edge.active {
                    path.push(curr_edge.src);

                    let src_neuron = self.nodes.get_mut(curr_edge.src.index())?;
                    src_neuron.error += curr_edge.weight * curr_error;

                    // add the weight step (gradient) * the currnet value to the weight to adjust the weight
                    // then update the connection so it knows if it should update the weight, or store the delta
                    let delta = match &self.trace_states {
                        Some(tracer) => step * tracer.neuron_activation(src_neuron.id),
                        None => step * src_neuron.activated_value
                    };

                    // Update edge
                    curr_edge.update(delta);
                }
            }
            // clear pending updates.
            edge_updates.clear();
        }

        // gather and return the output of the backwards pass
        let mut output = Vec::with_capacity(self.inputs.len());
        for x in self.inputs.iter() {
            let neuron = self.nodes.get_mut(x.index()).unwrap();
            let error = match &self.trace_states {
                Some(tracer) => neuron.error * tracer.neuron_activation(neuron.id),
                None => neuron.error * neuron.activated_value
            };
            neuron.error = 0.0;
            output.push(error);
        }
        // deduct the backprop index 
        if let Some(tracer) = &mut self.trace_states {
            tracer.index -= 1;
        }
        Some(output)
    }


    fn reset(&mut self) {
        if let Some(tracer) = &mut self.trace_states {
            tracer.reset();
        }
        self.reset_neurons();
    }


    /// add a tracer to the layer to keep track of historical meta data
    fn add_tracer(&mut self) {
        self.trace_states = Some(Tracer::new());
    }


    fn remove_tracer(&mut self) {
        self.trace_states = None;
    }



    fn as_ref_any(&self) -> &dyn Any
        where Self: Sized + 'static
    {
        self
    }


    fn as_mut_any(&mut self) -> &mut dyn Any
        where Self: Sized + 'static
    {
        self
    }


    fn shape(&self) -> (usize, usize) {
        (self.inputs.len(), self.outputs.len())
    }
}


impl Genome<Dense, NeatEnvironment> for Dense
    where Dense: Layer
{
    fn crossover(child: &Dense, parent_two: &Dense, env: Arc<RwLock<NeatEnvironment>>, crossover_rate: f32) -> Option<Dense> {
        let mut new_child = child.clone();
        let set = (*env).read().ok()?;
        let mut r = rand::thread_rng();
        if r.gen::<f32>() < crossover_rate {
            for edge in new_child.edges.iter_mut() {
                // if the edge is in both networks, then radnomly assign the weight to the edge
                // because we are already looping over the most fit parent, we only need to change the 
                // weight to the second parent if nessesary.
                if let Some(parent_edge) = parent_two.get_edge_by_innov(&edge.innov) {
                    if r.gen::<f32>() < 0.5 {
                        edge.weight = parent_edge.weight;
                    }

                    // if the edge is deactivated in either network and a random number is less than the 
                    // reactivate parameter, then reactiveate the edge and insert it back into the network
                    if (!edge.active || !parent_edge.active) && r.gen::<f32>() < set.reactivate? {
                        edge.link_nodes(&mut new_child.nodes);
                        edge.active = true;
                    }
                }
            }
        } else {
            // if a random number is less than the edit_weights parameter, then edit the weights of the network edges
            // add a possible new node to the network randomly 
            // attempt to add a new edge to the network, there is a chance this operation will add no edge
            if r.gen::<f32>() < set.weight_mutate_rate? {
                new_child.edit_weights(set.edit_weights?, set.weight_perturb?);
            }

            // if the layer is a dense pool then it can add nodes and connections to the layer as well
            if new_child.layer_type == LayerType::DensePool {
                if r.gen::<f32>() < set.new_node_rate? {
                    let act_func = *set.activation_functions.choose(&mut r)?;
                    if r.gen::<f32>() < set.recurrent_neuron_rate? {
                        new_child.add_node(act_func, NeuronDirection::Recurrent);
                    } else {
                        new_child.add_node(act_func, NeuronDirection::Forward);
                    }
                }
                if r.gen::<f32>() < set.new_edge_rate? {
                    new_child.add_edge();
                }
            }
        }
        Some(new_child)
    }



    fn distance(one: &Dense, two: &Dense, _: Arc<RwLock<NeatEnvironment>>) -> f32 {
        let mut similar = 0.0;
        for innov in one.edge_innov_map.keys() {
            if two.contains_edge(innov) {
                similar += 1.0;
            }
        }
        let one_score = similar / one.edges.len() as f32;
        let two_score = similar / two.edges.len() as f32;
        2.0 - (one_score + two_score)
    }
}


/// Implement partialeq for neat because if neat itself is to be used as a problem,
/// it must be able to compare one to another
impl PartialEq for Dense {
    fn eq(&self, other: &Self) -> bool {
        if self.edges.len() != other.edges.len() || self.nodes.len() != other.nodes.len() {
            return false;
        } 
        for (one, two) in self.edges.iter().zip(other.edges.iter()) {
            if one != two {
                return false;
            }
        }
        true
    }
}

/// Simple override of display for neat to debug a little cleaner 
impl fmt::Display for Dense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dense=[{}, {}]", self.nodes.len(), self.edges.len())
    }
}
