use std::fmt::Debug;

use super::{Direction, GraphNode, NodeIdx};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Strongly connected components, by Tarjan's algorithm (iterative, so deep graphs can't
/// overflow the stack). Used by [GraphPlan](super::compile::GraphPlan) to order a graph's nodes
/// and by graph transactions to mark which nodes sit on a cycle.
///
/// Fill `edges`/`edge_ranges` with each node's sources and call [Sccs::find], or let
/// [Sccs::find_in] do both from a graph's nodes. Edges point from a node to the nodes it reads,
/// so a component is emitted only after every component it reads from: `components` comes out
/// inputs first.
#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sccs {
    pub(super) edges: Vec<u32>,
    pub(super) edge_ranges: Vec<(u32, u32)>,
    pub(super) components: Vec<u32>,
    pub(super) component_ends: Vec<u32>,
    index: Vec<u32>,
    low: Vec<u32>,
    on_stack: Vec<bool>,
    stack: Vec<u32>,
    calls: Vec<(u32, u32)>,
}

impl Sccs {
    const UNVISITED: u32 = u32::MAX;

    pub(super) fn find(&mut self, n: usize) {
        self.index.clear();
        self.index.resize(n, Self::UNVISITED);
        self.low.clear();
        self.low.resize(n, 0);
        self.on_stack.clear();
        self.on_stack.resize(n, false);
        self.stack.clear();
        self.calls.clear();
        self.components.clear();
        self.component_ends.clear();

        let mut next = 0;
        for root in 0..n as u32 {
            if self.index[root as usize] != Self::UNVISITED {
                continue;
            }

            self.visit(root, &mut next);
            while let Some(&(v, edge)) = self.calls.last() {
                let (_, end) = self.edge_ranges[v as usize];
                if edge < end {
                    self.calls.last_mut().unwrap().1 += 1;
                    let w = self.edges[edge as usize];
                    if self.index[w as usize] == Self::UNVISITED {
                        self.visit(w, &mut next);
                    } else if self.on_stack[w as usize] {
                        self.low[v as usize] = self.low[v as usize].min(self.index[w as usize]);
                    }
                    continue;
                }

                self.calls.pop();
                if let Some(&(parent, _)) = self.calls.last() {
                    self.low[parent as usize] = self.low[parent as usize].min(self.low[v as usize]);
                }

                if self.low[v as usize] == self.index[v as usize] {
                    loop {
                        let w = self.stack.pop().unwrap();
                        self.on_stack[w as usize] = false;
                        self.components.push(w);
                        if w == v {
                            break;
                        }
                    }
                    self.component_ends.push(self.components.len() as u32);
                }
            }
        }
    }

    /// Components of `nodes`, following each node's `incoming()`, so members are node indices.
    pub(super) fn find_in<T>(&mut self, nodes: &[GraphNode<T>]) {
        self.edges.clear();
        self.edge_ranges.clear();
        self.edge_ranges.reserve(nodes.len());

        for node in nodes {
            let start = self.edges.len() as u32;
            self.edges
                .extend(node.incoming().iter().map(|src| src.as_usize() as u32));
            self.edge_ranges.push((start, self.edges.len() as u32));
        }

        self.find(nodes.len());
    }

    /// Each component's members, in emission order (inputs first).
    pub(super) fn iter(&self) -> impl Iterator<Item = &[u32]> {
        let mut start = 0;
        self.component_ends.iter().map(move |&end| {
            let members = &self.components[start as usize..end as usize];
            start = end;
            members
        })
    }

    /// Sorts each component's members ascending, in place.
    pub(super) fn sort_components(&mut self) {
        let mut start = 0;
        for &end in &self.component_ends {
            self.components[start as usize..end as usize].sort_unstable();
            start = end;
        }
    }

    /// A component is a cycle if it has more than one member, or its one member reads itself.
    pub(super) fn is_cycle(&self, members: &[u32]) -> bool {
        members.len() > 1 || {
            let v = members[0];
            let (start, end) = self.edge_ranges[v as usize];
            self.edges[start as usize..end as usize].contains(&v)
        }
    }

    /// Every node's [Direction] after [Sccs::find_in]: `Backward` if it sits on a cycle,
    /// `Forward` otherwise.
    pub(super) fn directions(&self) -> impl Iterator<Item = (NodeIdx, Direction)> + '_ {
        self.iter().flat_map(move |members| {
            let direction = if self.is_cycle(members) {
                Direction::Backward
            } else {
                Direction::Forward
            };

            members.iter().map(move |&m| (NodeIdx::new(m), direction))
        })
    }

    fn visit(&mut self, v: u32, next: &mut u32) {
        self.index[v as usize] = *next;
        self.low[v as usize] = *next;
        *next += 1;
        self.stack.push(v);
        self.on_stack[v as usize] = true;
        self.calls.push((v, self.edge_ranges[v as usize].0));
    }
}

impl Debug for Sccs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sccs")
            .field("components", &self.components)
            .field("component_ends", &self.component_ends)
            .finish()
    }
}
