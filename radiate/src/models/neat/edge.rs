use super::id::*;
use super::neuron::*;
use uuid::Uuid;

/// Edge is a connection between two nodes in the graph
///
/// Src is the innovation number of the node sending input through the network
/// dst is the innovation number of the node receiving the input from the src neuron
/// innov is the edge's unique innovation number for crossover and mutation
/// weight is the weight of the connection
/// active keeps track of if this edge is active or not, meaning it will be used
/// while feeding data through the network
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub innov: Uuid,
    pub src: NeuronId,
    pub dst: NeuronId,
    pub weight: f32,
    pub active: bool,
}

impl Edge {
    pub fn new(id: EdgeId, src: NeuronId, dst: NeuronId, weight: f32, active: bool) -> Self {
        Edge {
            id,
            src,
            dst,
            innov: Uuid::new_v4(),
            weight,
            active,
        }
    }

    /// update the weight of this edge connection
    #[inline]
    pub fn update(&mut self, delta: f32, nodes: &mut [Neuron]) {
        self.update_weight(self.weight + delta, nodes);
    }

    /// calculate the eligibility of this connection and store it for time series predictions
    #[inline]
    pub fn calculate(&self, val: f32) -> f32 {
        val * self.weight
    }

    /// update weight
    pub fn update_weight(&mut self, weight: f32, nodes: &mut [Neuron]) {
        self.weight = weight;
        nodes
            .get_mut(self.dst.index())
            .map(|x| x.update_incoming(self, weight));
    }

    /// Link edge src/dst nodes
    pub fn link_nodes(&self, nodes: &mut [Neuron]) {
        nodes
            .get_mut(self.src.index())
            .map(|x| x.add_outgoing(self.id));
        nodes
            .get_mut(self.dst.index())
            .map(|x| x.add_incoming(self));
    }

    /// Enable edge and link the nodes.
    pub fn enable(&mut self, nodes: &mut [Neuron]) {
        if self.active {
            // already active, nothing to do.
            return;
        }
        self.active = true;
        nodes
            .get_mut(self.src.index())
            .map(|x| x.add_outgoing(self.id));
        // For dst node, just re-enable the weight.
        // This allows for faster forward propagation.
        nodes
            .get_mut(self.dst.index())
            .map(|x| x.update_incoming(self, self.weight));
    }

    /// Disable edge and unlink the nodes.
    pub fn disable(&mut self, nodes: &mut [Neuron]) {
        self.active = false;
        if let Some(neuron) = nodes.get_mut(self.src.index()) {
            neuron.remove_outgoing(self.id)
        }
        // For dst node, just set the weight to zero.
        // This allows for faster forward propagation.
        if let Some(neuron) = nodes.get_mut(self.dst.index()) {
            neuron.update_incoming(self, 0.0)
        }
    }
}
