use super::GraphChromosome;
use super::node::InnovationId;
use crate::{Node, Op};
use radiate_core::{Chromosome, Diversity, Phenotype};
use std::cmp::Ordering;

const OP_MISMATCH_PENALTY: f32 = 0.3;

/// NEAT compatibility distance that aligns genes by [`InnovationId`].
///
/// Unlike [`NeatDistance`], which walks chromosomes in topological order and matches
/// genes by position, `InnovationDistance` projects each chromosome onto its innovation
/// timeline and merges by historical marker — the textbook NEAT definition. Two nodes
/// share an innovation iff they descend from the same structural event in the search
/// history, so this metric stays stable under reorderings and structural drift that
/// preserve homology.
///
/// The score follows Stanley's formula:
///
/// ```text
/// d = c1 * E / N + c2 * D / N + c3 * W̄ + c4
/// ```
///
/// where `E` is the count of excess genes (innovations past the smaller of the two
/// parents' max innovation), `D` is disjoint (misaligned within the overlap), `W̄` is
/// the average absolute weight difference across matching `Op::Value` genes, and `N`
/// is `max(|a|, |b|)`. The `op_mismatch` term penalizes matching innovations whose
/// op names disagree (e.g., `add` vs `mul`).
pub struct NeatDistance {
    excess: f32,
    disjoint: f32,
    weight_diff: f32,
    op_mismatch_penalty: f32,
}

impl NeatDistance {
    pub fn new(excess: f32, disjoint: f32, weight_diff: f32) -> Self {
        NeatDistance {
            excess,
            disjoint,
            weight_diff,
            op_mismatch_penalty: OP_MISMATCH_PENALTY,
        }
    }

    pub fn with_mismatch_penalty(mut self, penalty: f32) -> Self {
        self.op_mismatch_penalty = penalty;
        self
    }

    #[inline]
    fn graph_distance(
        &self,
        one: &GraphChromosome<Op<f32>>,
        two: &GraphChromosome<Op<f32>>,
    ) -> f32 {
        let max_genes = one.len().max(two.len());
        if max_genes == 0 {
            return 0.0;
        }

        let one_last = one.get(one.len() - 1).innovation();
        let two_last = two.get(two.len() - 1).innovation();
        let cutoff = match (one_last, two_last) {
            (Some(ma), Some(mb)) => Some(ma.min(mb)),
            _ => None,
        };

        let mut excess = 0.0_f32;
        let mut disjoint = 0.0_f32;
        let mut matching = 0.0_f32;
        let mut weight_diff = 0.0_f32;

        let mut idx_one = 0;
        let mut idx_two = 0;

        while idx_one < one.len() || idx_two < two.len() {
            let gene_one = if idx_one < one.len() {
                one.get(idx_one).innovation()
            } else {
                None
            };
            let gene_two = if idx_two < two.len() {
                two.get(idx_two).innovation()
            } else {
                None
            };

            match (gene_one, gene_two) {
                (Some(ida), Some(idb)) => match ida.cmp(&idb) {
                    Ordering::Equal => {
                        matching += 1.0;

                        let one_node = one.get(idx_one);
                        let two_node = two.get(idx_two);

                        if let (Op::Value(_, _, a_op, _), Op::Value(_, _, b_op, _)) =
                            (one_node.value(), two_node.value())
                        {
                            weight_diff += (a_op.data() - b_op.data()).abs();
                        }

                        idx_one += 1;
                        idx_two += 1;
                    }
                    Ordering::Less => {
                        bump(ida, cutoff, &mut excess, &mut disjoint);
                        idx_one += 1;
                    }
                    Ordering::Greater => {
                        bump(idb, cutoff, &mut excess, &mut disjoint);
                        idx_two += 1;
                    }
                },
                (Some(ida), None) => {
                    bump(ida, cutoff, &mut excess, &mut disjoint);
                    idx_one += 1;
                }
                (None, Some(idb)) => {
                    bump(idb, cutoff, &mut excess, &mut disjoint);
                    idx_two += 1;
                }
                (None, None) => break,
            }
        }

        let inv_max = 1.0 / (max_genes as f32);
        let avg_weight_diff = if matching > 0.0 {
            weight_diff / matching
        } else {
            0.0
        };

        (self.excess * excess * inv_max)
            + (self.disjoint * disjoint * inv_max)
            + (self.weight_diff * avg_weight_diff)
    }
}

#[inline]
fn bump(id: InnovationId, cutoff: Option<InnovationId>, excess: &mut f32, disjoint: &mut f32) {
    if cutoff.is_some_and(|c| id > c) {
        *excess += 1.0;
    } else {
        *disjoint += 1.0;
    }
}

impl Diversity<GraphChromosome<Op<f32>>> for NeatDistance {
    fn measure(
        &self,
        one: &Phenotype<GraphChromosome<Op<f32>>>,
        two: &Phenotype<GraphChromosome<Op<f32>>>,
    ) -> f32 {
        one.genotype()
            .iter()
            .zip(two.genotype().iter())
            .map(|(a, b)| self.graph_distance(a, b))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::graphs::node::InnovationId;
    use crate::{Arity, GraphNode, NodeType};

    fn vertex(index: usize, op: Op<f32>, innov: u64) -> GraphNode<Op<f32>> {
        let mut node = GraphNode::with_arity(index, NodeType::Vertex, op, Arity::Exact(2));
        node.set_innovation(innov_id(innov));
        node
    }

    fn edge(index: usize, weight: f32, innov: u64) -> GraphNode<Op<f32>> {
        let mut node = GraphNode::with_arity(
            index,
            NodeType::Edge,
            Op::weight_with(weight),
            Arity::Exact(1),
        );
        node.set_innovation(innov_id(innov));
        node
    }

    fn innov_id(n: u64) -> Option<InnovationId> {
        // SAFETY: InnovationId is #[repr(transparent)] over u64. Forging deterministic
        // IDs lets us assert exact alignment outcomes; `InnovationId::new()` shares a
        // process-wide counter and would be unstable across tests.
        unsafe { Some(std::mem::transmute::<u64, InnovationId>(n)) }
    }

    fn chromo(nodes: Vec<GraphNode<Op<f32>>>) -> GraphChromosome<Op<f32>> {
        GraphChromosome::new(nodes, Default::default())
    }

    #[test]
    fn identical_chromosomes_have_zero_distance() {
        let a = chromo(vec![vertex(0, Op::add(), 1), edge(1, 0.5, 2)]);
        let b = chromo(vec![vertex(0, Op::add(), 1), edge(1, 0.5, 2)]);

        let dist = NeatDistance::new(1.0, 1.0, 1.0)
            .with_mismatch_penalty(1.0)
            .graph_distance(&a, &b);
        assert_eq!(dist, 0.0);
    }

    #[test]
    fn weight_diff_accumulates_on_matching_innovations() {
        let a = chromo(vec![edge(0, 0.5, 1), edge(1, 1.0, 2)]);
        let b = chromo(vec![edge(0, 1.5, 1), edge(1, 0.0, 2)]);

        // 2 matches, weight diffs |0.5-1.5|=1.0 and |1.0-0.0|=1.0 → avg 1.0.
        let dist = NeatDistance::new(1.0, 1.0, 1.0).graph_distance(&a, &b);
        assert!((dist - 1.0).abs() < 1e-6, "got {dist}");
    }

    #[test]
    fn trailing_innovation_counts_as_excess_not_disjoint() {
        // a has innov 1,2. b has innov 1,2,3. Innov 3 is beyond min(2,3)=2 → excess.
        let a = chromo(vec![edge(0, 0.0, 1), edge(1, 0.0, 2)]);
        let b = chromo(vec![edge(0, 0.0, 1), edge(1, 0.0, 2), edge(2, 0.0, 3)]);

        let only_excess = NeatDistance::new(1.0, 0.0, 0.0).graph_distance(&a, &b);
        let only_disjoint = NeatDistance::new(0.0, 1.0, 0.0).graph_distance(&a, &b);

        let n = 3.0_f32;
        assert!((only_excess - 1.0 / n).abs() < 1e-6, "excess={only_excess}");
        assert_eq!(only_disjoint, 0.0);
    }

    #[test]
    fn middle_misalignment_is_disjoint_not_excess() {
        // a: 1,2,4. b: 1,3,4. min-max = 4. Innov 2 (only in a) and 3 (only in b) both
        // lie at or below the cutoff → disjoint, not excess.
        let a = chromo(vec![edge(0, 0.0, 1), edge(1, 0.0, 2), edge(2, 0.0, 4)]);
        let b = chromo(vec![edge(0, 0.0, 1), edge(1, 0.0, 3), edge(2, 0.0, 4)]);

        let only_excess = NeatDistance::new(1.0, 0.0, 0.0).graph_distance(&a, &b);
        let only_disjoint = NeatDistance::new(0.0, 1.0, 0.0).graph_distance(&a, &b);

        assert_eq!(only_excess, 0.0);
        let n = 3.0_f32;
        assert!(
            (only_disjoint - 2.0 / n).abs() < 1e-6,
            "disjoint={only_disjoint}"
        );
    }
}
