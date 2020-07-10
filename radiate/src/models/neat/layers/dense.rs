extern crate rand;

use std::fmt;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rand::Rng;
use rand::seq::SliceRandom;
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use super::{
    layertype::LayerType,
    layer::Layer,
    vectorops
};
use super::super::{
    id::*,
    neuron::Neuron,
    edge::Edge,
    tracer::Tracer,
    neatenv::NeatEnvironment,
    neurontype::NeuronType,
    activation::Activation,
    direction::NeuronDirection
};

use crate::Genome;



#[derive(Debug)]
pub struct Dense {
    pub inputs: Vec<NeuronId>,
    pub outputs: Vec<NeuronId>,
    pub nodes: HashMap<NeuronId, Neuron>,
    pub edges: HashMap<EdgeId, Edge>,
    pub trace_states: Option<Tracer>,
    pub layer_type: LayerType,
    pub activation: Activation
}



impl Dense {

    /// create a new fully connected dense layer.
    /// Each input is connected to each output with a randomly generated weight attached to the connection
    #[inline]
    pub fn new(num_in: u32, num_out: u32, layer_type: LayerType, activation: Activation) -> Self {
        let mut layer = Dense {
            inputs: (0..num_in)
                .into_iter()
                .map(|_| NeuronId::new())
                .collect(),
            outputs: (0..num_out)
                .into_iter()
                .map(|_| NeuronId::new())
                .collect(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            trace_states: None, 
            layer_type,
            activation
        };

        for innov in layer.inputs.iter() {
            layer.nodes.insert(*innov, Neuron::new(*innov, NeuronType::Input, activation, NeuronDirection::Forward));
        }
        for innov in layer.outputs.iter() {
            layer.nodes.insert(*innov, Neuron::new(*innov, NeuronType::Output, activation, NeuronDirection::Forward));
        }

        let mut r = rand::thread_rng();
        for i in layer.inputs.iter() {
            for j in layer.outputs.iter() {
                let weight = r.gen::<f32>() * 2.0 - 1.0;
                let new_edge = Edge::new(*i, *j, EdgeId::new(), weight, true);
                layer.nodes.get_mut(i).map(|x| x.outgoing.push(new_edge.innov));
                layer.nodes.get_mut(j).map(|x| x.incoming.insert(new_edge.innov, None));
                layer.edges.insert(new_edge.innov, new_edge);
            }
        }

        layer
    }


    /// This method is needed because we can't have multiple mutable
    /// references to a HashMap.
    #[inline]
    fn link_nodes(&mut self, sending: &NeuronId, receiving: &NeuronId, edge: &EdgeId) {
        self.nodes.get_mut(sending).map(|x| x.outgoing.push(*edge));
        self.nodes.get_mut(receiving).map(|x| x.incoming.insert(*edge, None));
    }

    /// reset all the neurons in the network so they can be fed forward again
    #[inline]
    fn reset_neurons(&mut self) {
        for val in self.nodes.values_mut() {
            val.reset_neuron();
        }
    }



    /// get the outputs from the layer in a vec form
    #[inline]
    pub fn get_outputs(&self) -> Option<Vec<f32>> {
        let result = self.outputs
            .iter()
            .map(|x| {
                self.nodes.get(x).unwrap().activated_value
            })
            .collect::<Vec<_>>();
        Some(result)
    }



    /// Add a node to the network by getting a random edge 
    /// and inserting the new node inbetween that edge's source
    /// and destination nodes. The old weight is pushed forward 
    /// while the new weight is randomly chosen and put between the 
    /// old source node and the new node
    #[inline]
    pub fn add_node(&mut self, activation: Activation, direction: NeuronDirection) {
        // create a new node to insert inbetween the sending and receiving nodes 
        let mut new_node = Neuron::new(NeuronId::new(), NeuronType::Hidden, activation, direction);

        // get an edge to insert the node into
        let curr_edge = {
          let edge = self.edges.get_mut(&self.random_edge()).unwrap();
          // disable the edge
          edge.active = false;
          edge.clone()
        };

        // create two new edges that connect the src and the new node and the 
        // new node and dst, then disable the current edge 
        let incoming = Edge::new(curr_edge.src, new_node.innov, EdgeId::new(), 1.0, true);
        let outgoing = Edge::new(new_node.innov, curr_edge.dst, EdgeId::new(), curr_edge.weight, true);

        // Update sending node.
        {
            let sending = self.nodes.get_mut(&curr_edge.src).unwrap();
            // remove old edge to receiving node
            sending.outgoing.retain(|x| x != &(curr_edge.innov));

            // add new edge to new node.
            sending.outgoing.push(incoming.innov);
            new_node.incoming.insert(incoming.innov, None);
        }
        // Update receiving node.
        {
            let receiving = self.nodes.get_mut(&curr_edge.dst).unwrap();
            // remove old edge from sending node.
            receiving.incoming.remove(&curr_edge.innov);

            // add the new edge from new node.
            new_node.outgoing.push(outgoing.innov);
            receiving.incoming.insert(outgoing.innov, None);
        }

        // add the new nodes and the new edges to the network
        self.edges.insert(incoming.innov, incoming);
        self.edges.insert(outgoing.innov, outgoing);
        self.nodes.insert(new_node.innov, new_node);
    }



    /// add a connection to the network. Randomly get a sending node that cannot 
    /// be an output and a receiving node which is not an input node, the validate
    /// that the desired connection can be made. If it can be, make the connection
    /// with a weight of .5 in order to minimally impact the network 
    #[inline]
    pub fn add_edge(&mut self) {
        // get a valid sending neuron
        let sending = self.random_node_not_of_type(NeuronType::Output);
        // get a vaild receiving neuron
        let receiving = self.random_node_not_of_type(NeuronType::Input);

        // determine if the connection to be made is valid 
        if self.valid_connection(sending, receiving) {
            // if the connection is valid, make it and wire the nodes to each
            let mut r = rand::thread_rng();
            let new_edge = Edge::new(sending, receiving, EdgeId::new(), r.gen::<f32>(), true);
            self.link_nodes(&sending, &receiving, &new_edge.innov);

            // add the new edge to the network
            self.edges.insert(new_edge.innov, new_edge);
        }
    }



    /// Test whether the desired connection is valid or not by testing to see if 
    /// 1.) it is recursive
    /// 2.) the connection already exists
    /// 3.) the desired connection would create a cycle in the graph
    /// if these are all false, then the connection can be made
    #[inline]
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
    #[inline]
    fn cyclical(&self, sending: NeuronId, receiving: NeuronId) -> bool {
        let recv_node = self.nodes.get(&receiving).unwrap();
        // dfs stack which gets the receiving Neuron<dyn neurons> outgoing connections
        let mut stack = recv_node.outgoing
            .iter()
            .map(|x| self.edges.get(x).unwrap().dst)
            .collect::<Vec<_>>();

        // while the stack still has nodes, continue
        while stack.len() > 0 {
            // if the current node is the same as the sending, this would cause a cycle
            // else add all the current node's outputs to the stack to search through
            let curr = self.nodes.get(&stack.pop().unwrap()).unwrap();
            if curr.innov == sending {
                return true;
            }
            for i in curr.outgoing.iter() {
                stack.push(self.edges.get(i).unwrap().dst);
            }
        }
        false
    }



    /// check if the desired connection already exists within he network, if it does then
    /// we should not be creating the connection.
    #[inline]
    fn exists(&self, sending: NeuronId, receiving: NeuronId) -> bool {
        for val in self.edges.values() {
            if val.src == sending && val.dst == receiving {
                return true
            }
        }
        false
    }



    /// get a random node from the network
    #[inline]
    fn random_node(&self) -> &Neuron {
        let index = rand::thread_rng().gen_range(0, self.nodes.len());
        let node = self.nodes.values().nth(index)
          .expect("Failed to get random node");
        return node;
    }

    /// get a random node from the network not of the specific type
    #[inline]
    fn random_node_not_of_type(&self, node_type: NeuronType) -> NeuronId {
        loop {
            let node = self.random_node();
            if node.neuron_type != node_type {
                break node.innov;
            }
        }
    }


    /// get a random connection from the network - hashmap does not have an idomatic
    /// way to do this so this is a workaround. Returns the innovatio number of the 
    /// edge in order to satisfy rust borrowing rules
    #[inline]
    fn random_edge(&self) -> EdgeId {
        let index = rand::thread_rng().gen_range(0, self.edges.len());
        for (i, (innov, _)) in self.edges.iter().enumerate() {
            if i == index {
                return *innov;
            }
        }
        panic!("Failed to get random edge");
    }



    /// give input data to the input nodes in the network and return a vec
    /// that holds the innovation numbers of the input nodes for a dfs traversal 
    /// to feed forward those inputs through the network
    #[inline]
    fn give_inputs(&mut self, data: &Vec<f32>) -> Vec<NeuronId> {
        assert!(data.len() == self.inputs.len());
        let mut ids = Vec::with_capacity(self.inputs.len());
        for (node_innov, input) in self.inputs.iter().zip(data.iter()) {
            let node = self.nodes.get_mut(node_innov).unwrap();
            node.activated_value = *input;
            ids.push(node.innov);
        }
        ids
    }



    /// Edit the weights in the network randomly by either uniformly perturbing
    /// them, or giving them an entire new weight all together
    #[inline]
    fn edit_weights(&mut self, editable: f32, size: f32) {
        let mut r = rand::thread_rng();
        for (_, edge) in self.edges.iter_mut() {
            if r.gen::<f32>() < editable {
                edge.weight = r.gen::<f32>();
            } else {
                edge.weight *= r.gen_range(-size, size);
            }
        }
        for node in self.nodes.values_mut() {
            if r.gen::<f32>() < editable {
                node.bias = r.gen::<f32>();
            } else {
                node.bias *= r.gen_range(-size, size);
            }
        }
    }



    /// get the states of the output neurons. This allows softmax and other specific actions to 
    /// be taken where knowledge of more than just the immediate neuron's state must be known
    #[inline]
    pub fn get_output_states(&self) -> Vec<f32> {
        self.outputs
            .iter()
            .map(|x| {
                let output_neuron = self.nodes.get(x).unwrap();
                output_neuron.current_state
            })
            .collect::<Vec<_>>()
    }



    /// Because the output neurons might need to be seen togehter, this must be called to 
    /// set their values before finishing the feed forward function
    #[inline]
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
            let curr_neuron = self.nodes.get_mut(neuron_id).unwrap();
            curr_neuron.activated_value = act[i];
            curr_neuron.deactivated_value = d_act[i];
        }
    }



    /// take a snapshot of the neuron's values at this time step if trace is enabled
    #[inline]
    pub fn update_traces(&mut self) {
        if let Some(tracer) = &mut self.trace_states {
            for (n_id, node) in self.nodes.iter() {
                tracer.update_neuron_activation(n_id, node.activated_value);
                tracer.update_neuron_derivative(n_id, node.deactivated_value);
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
    #[inline]
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
        while path.len() > 0 {

            // remove the top elemet to propagate it's value
            let curr_node = self.nodes.get(&path.pop()?)?;

            // no node should be in the path if it's value has not been set 
            // iterate through the current nodes outgoing connections 
            // to get its value and give that value to it's connected node
            for edge_innov in curr_node.outgoing.iter() {

                // if the currnet edge is active in the network, we can propagate through it
                let curr_edge = self.edges.get(edge_innov)?;
                if curr_edge.active {
                    let activated_value = curr_edge.calculate(curr_node.activated_value);
                    node_updates.push((curr_edge.dst, curr_edge.innov, activated_value));
                }
            }

            // apply pending node updates.
            for &(dst, edge, value) in node_updates.iter() {
                let receiving_node = self.nodes.get_mut(&dst)?;
                receiving_node.incoming.insert(edge, Some(value));

                // if the node can be activated, activate it and store it's value
                // only activated nodes can be added to the path, so if it's activated
                // add it to the path so the values can be propagated through the network
                if receiving_node.is_ready() {
                    receiving_node.activate();
                    path.push(receiving_node.innov);
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
    #[inline]
    fn backward(&mut self, error: &Vec<f32>, learning_rate: f32) -> Option<Vec<f32>> {
        // feed forward the input data to get the output in order to compute the error of the network
        // create a dfs stack to step backwards through the network and compute the error of each neuron
        // then insert that error in a hashmap to keep track of innov of the neuron and it's error 
        let mut path = Vec::with_capacity(self.inputs.len());
        for (index, innov) in self.outputs.iter().enumerate() {
            let node = self.nodes.get_mut(innov).unwrap();
            node.error = error[index];
            path.push(*innov);
        }

        // edge_updates is used to split immutable & mutable node access.
        let mut edge_updates = Vec::with_capacity(self.inputs.len());

        // step through the network backwards and adjust the weights
        while path.len() > 0 {
            // get the current node and it's error 
            let curr_node = self.nodes.get_mut(&path.pop()?)?;
            let curr_error = curr_node.error;
            let step = match &self.trace_states {
                Some(tracer) => curr_error * tracer.neuron_derivative(curr_node.innov),
                None => curr_error * curr_node.deactivated_value
            } * learning_rate;

            // reset the nodes error if it isnt an input node 
            if curr_node.neuron_type != NeuronType::Input {
                curr_node.bias += learning_rate * curr_error;
                curr_node.error = 0.0;
            }

            // iterate through each of the incoming edes to this neuron and adjust it's weight
            // and add it's error to the errros map
            for incoming_edge_innov in curr_node.incoming.keys() {
                edge_updates.push(*incoming_edge_innov);
            }

            // apply pending edge updates.
            for incoming_edge_innov in edge_updates.iter() {
                let curr_edge = self.edges.get_mut(incoming_edge_innov)?;

                // if the current edge is active, then it is contributing to the error and we need to adjust it
                if curr_edge.active {
                    path.push(curr_edge.src);

                    let src_neuron = self.nodes.get_mut(&curr_edge.src)?;
                    src_neuron.error += curr_edge.weight * curr_error;

                    // add the weight step (gradient) * the currnet value to the weight to adjust the weight
                    // then update the connection so it knows if it should update the weight, or store the delta
                    let delta = match &self.trace_states {
                        Some(tracer) => step * tracer.neuron_activation(src_neuron.innov),
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
            let neuron = self.nodes.get_mut(x).unwrap();
            let error = match &self.trace_states {
                Some(tracer) => neuron.error * tracer.neuron_activation(neuron.innov),
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
            let mut links = Vec::with_capacity(new_child.edges.len());
            for (innov, edge) in new_child.edges.iter_mut() {
                // if the edge is in both networks, then radnomly assign the weight to the edge
                // because we are already looping over the most fit parent, we only need to change the 
                // weight to the second parent if nessesary.
                if parent_two.edges.contains_key(innov) {
                    if r.gen::<f32>() < 0.5 {
                        edge.weight = parent_two.edges.get(innov)?.weight;
                    }

                    // if the edge is deactivated in either network and a random number is less than the 
                    // reactivate parameter, then reactiveate the edge and insert it back into the network
                    if (!edge.active || !parent_two.edges.get(innov)?.active) && r.gen::<f32>() < set.reactivate? {
                        links.push((edge.src, edge.dst, *innov));
                        edge.active = true;
                    }
                }
            }
            for (src, dst, edge) in links.iter() {
                new_child.link_nodes(src, dst, edge);
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
        for (innov, _) in one.edges.iter() {
            if two.edges.contains_key(innov) {
                similar += 1.0;
            }
        }
        let one_score = similar / one.edges.len() as f32;
        let two_score = similar / two.edges.len() as f32;
        2.0 - (one_score + two_score)
    }
}




/// Implement clone for the neat neural network in order to facilitate 
/// proper crossover and mutation for the network
impl Clone for Dense {
    fn clone(&self) -> Self {
        Dense {
            inputs: self.inputs
                .iter()
                .map(|x| *x)
                .collect(),
            outputs: self.outputs
                .iter() 
                .map(|x| *x)
                .collect(),
            nodes: self.nodes
                .iter()
                .map(|(key, val)| {
                    (*key, val.clone())
                })
                .collect(),
            edges: self.edges
                .iter()
                .map(|(key, val)| {
                    (*key, val.clone())
                })
                .collect(),
            trace_states: self.trace_states.clone(),
            layer_type: self.layer_type.clone(),
            activation: self.activation.clone()
        }
    }
}

/// Implement partialeq for neat because if neat itself is to be used as a problem,
/// it must be able to compare one to another
impl PartialEq for Dense {
    fn eq(&self, other: &Self) -> bool {
        if self.edges.len() != other.edges.len() || self.nodes.len() != other.nodes.len() {
            return false;
        } 
        for (one, two) in self.edges.values().zip(other.edges.values()) {
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



/// manually implement serialize for dense because it uses raw pointrs so it cannot be 
/// derived due to there being no way to serialie and deserailize raw pointers
impl Serialize for Dense {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Dense", 7)?;
        let n = self.nodes
            .iter()
            .map(|x| (x.0, x.1.clone_with_values()) )
            .collect::<HashMap<_, _>>();
        s.serialize_field("inputs", &self.inputs)?;
        s.serialize_field("outputs", &self.outputs)?;
        s.serialize_field("nodes", &n)?;
        s.serialize_field("edges", &self.edges)?;
        s.serialize_field("trace_states", &self.trace_states)?;
        s.serialize_field("layer_type", &self.layer_type)?;
        s.serialize_field("activation", &self.activation)?;
        s.end()
    }
}



/// implement deserialize for dense layer - because the layer uses raw pointers, this needs to be implemented manually 
/// which is kinda a pain in the ass but this is the only  one that needs to be done manually - everything else is derived
impl<'de> Deserialize<'de> for Dense {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {

        enum Field { Inputs, Outputs, Nodes, Edges, TraceStates, LayerType, Activation };


        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "inputs" => Ok(Field::Inputs),
                            "outputs" => Ok(Field::Outputs),
                            "nodes" => Ok(Field::Nodes),
                            "edges" => Ok(Field::Edges),
                            "trace_states" => Ok(Field::TraceStates),
                            "layer_type" => Ok(Field::LayerType),
                            "activation" => Ok(Field::Activation),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DenseVisitor;

        impl<'de> Visitor<'de> for DenseVisitor {
            type Value = Dense;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Dense")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Dense, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let inputs = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let outputs = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let neurons: HashMap<NeuronId, Neuron> = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let edges = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let trace_states = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let layer_type = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let activation = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let nodes = neurons
                    .iter()
                    .map(|(k, v)| {
                        (k.clone(), v.clone_with_values())
                    })
                    .collect::<HashMap<_, _>>();

                let dense = Dense {
                    inputs, outputs, nodes, edges, trace_states, layer_type, activation
                };

                Ok(dense)
            }

            fn visit_map<V>(self, mut map: V) -> Result<Dense, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut inputs = None;
                let mut outputs = None;
                let mut nodes = None;
                let mut edges = None;
                let mut trace_states = None;
                let mut layer_type = None;
                let mut activation = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Inputs => {
                            if inputs.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            inputs = Some(map.next_value()?);
                        }
                        Field::Outputs => {
                            if outputs.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            outputs = Some(map.next_value()?);
                        },
                        Field::Nodes => {
                            if nodes.is_some() {
                                return Err(de::Error::duplicate_field("nodes"));
                            }
                            let temp: HashMap<NeuronId, Neuron> = map.next_value()?;
                            nodes = Some(temp
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone_with_values()))
                                .collect::<HashMap<_, _>>());
                        },
                        Field::Edges => {
                            if edges.is_some() {
                                return Err(de::Error::duplicate_field("edges"));
                            }
                            edges = Some(map.next_value()?);
                        },
                        Field::TraceStates => {
                            if trace_states.is_some() {
                                return Err(de::Error::duplicate_field("nodes"));
                            }
                            trace_states = Some(map.next_value()?);
                        },
                        Field::LayerType => {
                            if layer_type.is_some() {
                                return Err(de::Error::duplicate_field("nodes"));
                            }
                            layer_type = Some(map.next_value()?);
                        },
                        Field::Activation => {
                            if activation.is_some() {
                                return Err(de::Error::duplicate_field("nodes"));
                            }
                            activation = Some(map.next_value()?);
                        }
                    }
                }

                let inputs = inputs.ok_or_else(|| de::Error::missing_field("secs"))?;
                let outputs = outputs.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let nodes = nodes.ok_or_else(|| de::Error::missing_field("nodes"))?;
                let edges = edges.ok_or_else(|| de::Error::missing_field("edges"))?;
                let trace_states = trace_states.ok_or_else(|| de::Error::missing_field("trace_states"))?;
                let layer_type = layer_type.ok_or_else(|| de::Error::missing_field("layer_type"))?;
                let activation = activation.ok_or_else(|| de::Error::missing_field("activation"))?;

                let dense = Dense {
                    inputs, outputs, nodes, edges, trace_states, layer_type, activation 
                };
                Ok(dense)
            }
        }

        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Dense", FIELDS, DenseVisitor)
    }
}

