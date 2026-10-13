use super::{Tree, TreeChromosome, TreeNode};
use radiate_core::{AlterContext, Expr, Mutate, RateSet, SmallStr, Valid, random_provider};

const DEFAULT_MAX_DEPTH: usize = 2;
const SUBTREE_REJECTED: SmallStr = SmallStr::from_static("mutator.subtree.rejected");

#[derive(Clone, Debug)]
pub struct HoistMutator {
    rate: Expr,
}

impl HoistMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        HoistMutator { rate: rate.into() }
    }
}

impl<T> Mutate<TreeChromosome<T>> for HoistMutator
where
    T: Clone + PartialEq,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    fn mutate_chromosome(
        &mut self,
        chromosome: &mut TreeChromosome<T>,
        ctx: &mut AlterContext,
    ) -> usize {
        if !random_provider::bool(ctx.rate()) {
            return 0;
        }

        let root = chromosome.root_mut();
        let root_size = root.size();
        let rand_index = random_provider::range(0..root_size);

        if rand_index < 1 {
            return 0;
        }

        if let Some(rand_node) = root.get_mut(rand_index) {
            if rand_node.children().is_none() {
                return 0;
            }

            let child_idx = random_provider::range(0..rand_node.children().map_or(0, |c| c.len()));
            let mut child = rand_node.detach(child_idx);

            return if let Some(child) = child.as_mut() {
                std::mem::swap(rand_node, child);
                1
            } else {
                0
            };
        }

        0
    }
}

/// Koza-style subtree mutation: replaces a random subtree with a newly grown one.
///
/// With probability `rate` per chromosome, picks a random non-root node and replaces it (and
/// everything below it) with a subtree grown from the chromosome's [NodeSet](crate::NodeSet),
/// of a depth drawn uniformly from `0..=max_depth`. Depth 0 is a single leaf.
///
/// ```text
///   add(mul(x0, x1), x2)    pick mul(x0, x1), grow depth 1    add(log(x1), x2)
/// ```
///
/// The root is never replaced, so a [NodeType::Root](crate::NodeType::Root) op is kept. If the
/// new tree breaks the chromosome's constraint (e.g. a size limit), the old subtree is put back
/// and the mutation counts as rejected.
#[derive(Clone, Debug)]
pub struct SubtreeMutator {
    rate: Expr,
    max_depth: usize,
}

impl SubtreeMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        SubtreeMutator {
            rate: rate.into(),
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// The deepest subtree to grow. Defaults to 2.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }
}

impl<T> Mutate<TreeChromosome<T>> for SubtreeMutator
where
    T: Clone + PartialEq + Default,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    fn mutate_chromosome(
        &mut self,
        chromosome: &mut TreeChromosome<T>,
        ctx: &mut AlterContext,
    ) -> usize {
        if !random_provider::bool(ctx.rate()) {
            return 0;
        }

        let size = chromosome.root().size();
        let Some(store) = chromosome.get_store().filter(|_| size > 1) else {
            return 0;
        };

        let index = random_provider::range(1..size);
        let depth = random_provider::range(0..self.max_depth + 1);
        let Some(subtree) = Tree::grow(depth, &store) else {
            return 0;
        };

        replace_or_rollback(chromosome, index, subtree, ctx, SUBTREE_REJECTED)
    }
}

/// Replaces the node at pre-order `index` with `replacement`. If the chromosome is no longer
/// valid (e.g. it broke a size constraint), puts the old node back and records `rejected`.
/// Returns 1 if the replacement stuck, 0 otherwise.
fn replace_or_rollback<T>(
    chromosome: &mut TreeChromosome<T>,
    index: usize,
    replacement: TreeNode<T>,
    ctx: &mut AlterContext,
    rejected: SmallStr,
) -> usize {
    let Some(node) = chromosome.root_mut().get_mut(index) else {
        return 0;
    };

    let previous = std::mem::replace(node, replacement);

    if chromosome.is_valid() {
        return 1;
    }

    if let Some(node) = chromosome.root_mut().get_mut(index) {
        *node = previous;
    }

    ctx.upsert(rejected, 1);

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::{NodeSet, NodeType, Op, TreeNode};
    use radiate_core::alter::AlterUpdates;

    fn store() -> NodeSet<Op<f32>> {
        vec![
            (NodeType::Root, vec![Op::sum()]),
            (
                NodeType::Vertex,
                vec![Op::add(), Op::mul(), Op::log(), Op::sigmoid()],
            ),
            (NodeType::Leaf, vec![Op::var(0), Op::var(1), Op::erc()]),
        ]
        .into()
    }

    /// A random tree of `depth`, optionally constrained to at most `max_size` nodes.
    fn chromosome(depth: usize, max_size: Option<usize>) -> TreeChromosome<Op<f32>> {
        let root = Tree::with_depth(depth, store()).take_root().unwrap();
        TreeChromosome::new(vec![root], Some(store()), max_size.map(|m| m as u32))
    }

    /// Mutates `chromosome` `times` times at `rate`, returning (applied, rejected).
    fn mutate(
        mutator: &mut impl Mutate<TreeChromosome<Op<f32>>>,
        chromosome: &mut TreeChromosome<Op<f32>>,
        rate: f32,
        times: usize,
    ) -> (usize, usize) {
        let mut applied = 0;
        let mut rejected = 0;
        for _ in 0..times {
            let mut updates = AlterUpdates::default();
            let mut ctx = AlterContext::new(&mut updates, 0, rate, &[]);
            let n = mutator.mutate_chromosome(chromosome, &mut ctx);
            applied += n;
            rejected += (n == 0) as usize;
        }
        (applied, rejected)
    }

    #[test]
    fn test_subtree_keeps_root_and_valid_arity() {
        random_provider::scoped_seed(3, || {
            let mut chromosome = chromosome(3, None);
            let root = chromosome.root().value().clone();
            let mut mutator = SubtreeMutator::new(1.0);

            for _ in 0..200 {
                let before = chromosome.root().height();
                let (applied, _) = mutate(&mut mutator, &mut chromosome, 1.0, 1);

                assert_eq!(applied, 1);
                assert_eq!(chromosome.root().value(), &root);
                assert!(chromosome.is_valid());
                assert!(chromosome.root().height() <= before + DEFAULT_MAX_DEPTH);
            }
        });
    }

    #[test]
    fn test_subtree_respects_constraint() {
        random_provider::scoped_seed(5, || {
            let mut chromosome = chromosome(2, Some(12));
            assert!(chromosome.is_valid());

            let mut mutator = SubtreeMutator::new(1.0).with_max_depth(3);
            let mut rejected = 0;
            for _ in 0..500 {
                rejected += mutate(&mut mutator, &mut chromosome, 1.0, 1).1;
                assert!(chromosome.root().size() <= 12);
            }

            // Depth-3 subtrees regularly overflow a 12-node limit, so some must be rolled back.
            assert!(rejected > 0);
        });
    }

    #[test]
    fn test_subtree_zero_rate_and_single_node_are_untouched() {
        random_provider::scoped_seed(7, || {
            let mut mutator = SubtreeMutator::new(1.0);

            let mut chromosome = chromosome(3, None);
            let before = chromosome.root().clone();
            assert_eq!(mutate(&mut mutator, &mut chromosome, 0.0, 50), (0, 50));
            assert!(chromosome.root() == &before);

            let mut single =
                TreeChromosome::new(vec![TreeNode::new(Op::var(0))], Some(store()), None);
            assert_eq!(mutate(&mut mutator, &mut single, 1.0, 50), (0, 50));
            assert_eq!(single.root().size(), 1);
        });
    }
}
