
use uuid::Uuid;
use super::id::*;
use super::neuron::*;

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
    pub active: bool
}

impl Edge {
    pub fn new(id: EdgeId, src: NeuronId, dst: NeuronId, weight: f32, active: bool) -> Self {
        Edge {
            id,
            src,
            dst,
            innov: Uuid::new_v4(),
            weight,
            active
        }
    }

    /// update the weight of this edge connection
    #[inline]
    pub fn update(&mut self, delta: f32) {
        self.weight += delta;
    }

    /// calculate the eligibility of this connection and store it for time series predictions
    #[inline]
    pub fn calculate(&self, val: f32) -> f32 {
        val * self.weight
    }

    /// Link edge src/dst nodes
    pub fn link_nodes(&self, nodes: &mut Vec<Neuron>) {
        nodes.get_mut(self.src.index()).map(|x| x.add_outgoing(self.id));
        nodes.get_mut(self.dst.index()).map(|x| x.add_incoming(self.id));
    }

    /// Disable edge and unlink the nodes.
    pub fn disable(&mut self, nodes: &mut Vec<Neuron>) {
        self.active = false;
        nodes.get_mut(self.src.index()).map(|x| x.remove_outgoing(self.id));
        nodes.get_mut(self.dst.index()).map(|x| x.remove_incoming(self.id));
    }
}
