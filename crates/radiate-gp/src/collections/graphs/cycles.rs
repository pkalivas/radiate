#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Strongly connected components, by Tarjan's algorithm (iterative, so deep graphs can't
/// overflow the stack). Used by [GraphPlan](super::compile::GraphPlan) to order a graph's nodes
/// and by graph transactions to mark which nodes sit on a cycle.
///
/// Fill `edges`/`edge_ranges` with each node's sources (or use [Sccs::find_in]), then call
/// [Sccs::find]. Edges
/// point from a node to the nodes it reads, so a component is emitted only after every
/// component it reads from: `components` comes out inputs first.
#[derive(Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) struct Sccs {
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

    fn visit(&mut self, v: u32, next: &mut u32) {
        self.index[v as usize] = *next;
        self.low[v as usize] = *next;
        *next += 1;
        self.stack.push(v);
        self.on_stack[v as usize] = true;
        self.calls.push((v, self.edge_ranges[v as usize].0));
    }
}
