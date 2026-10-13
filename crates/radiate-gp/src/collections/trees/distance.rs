use super::{TreeChromosome, TreeNode};
use crate::node::Node;
use radiate_core::{Phenotype, distance::Distance};

const DEFAULT_K: f32 = 2.0;

/// Structural distance between trees, based on Ekárt & Németh's structural alignment distance
/// ("Selection based on the Pareto nondomination criterion for controlling code growth in
/// genetic programming", 2001), normalized to `[0, 1]`.
///
/// The two trees are laid over each other from the root, matching children by position, and
/// compared node by node. Each level down counts `1 / k` as much as the level above, so a
/// different root weighs more than a different leaf:
///
/// ```text
///   cost(node in both, same value)       = 0
///   cost(node in both, different value)  = 2     one removed, one added
///   cost(node in only one tree)          = 1
///
///   distance = Σ cost · (1/k)^depth  /  Σ (1/k)^depth over every node of both trees
/// ```
///
/// So identical trees are 0, and trees with no matching value at any aligned position are 1.
/// It runs in time linear in the trees' size and needs nothing but `T: PartialEq`, which makes
/// it cheap enough for speciation. For multi-tree genotypes (one tree per chromosome) the
/// distance is the average over tree pairs, a tree without a counterpart counting as 1.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeDistance {
    k: f32,
}

impl TreeDistance {
    pub fn new() -> Self {
        TreeDistance { k: DEFAULT_K }
    }

    /// How fast deeper levels fade: each level counts `1 / k` as much as the one above.
    /// Defaults to 2. `k = 1` weighs every node equally.
    ///
    /// # Panics
    /// If `k` is less than 1.
    pub fn with_k(mut self, k: f32) -> Self {
        assert!(k >= 1.0, "k must be at least 1, got {k}");
        self.k = k;
        self
    }

    /// The distance between two trees, in `[0, 1]`.
    pub fn between<T: PartialEq>(&self, one: &TreeNode<T>, two: &TreeNode<T>) -> f32 {
        let (cost, weight_one, weight_two) = self.walk(Some(one), Some(two));
        let total = weight_one + weight_two;

        if total > 0.0 { cost / total } else { 0.0 }
    }

    /// Returns `(cost, weight of one, weight of two)` for the overlaid subtrees, each scaled
    /// as if this level were the root.
    fn walk<T: PartialEq>(
        &self,
        one: Option<&TreeNode<T>>,
        two: Option<&TreeNode<T>>,
    ) -> (f32, f32, f32) {
        match (one, two) {
            (None, None) => (0.0, 0.0, 0.0),
            (Some(node), None) => {
                let weight = self.weight(node);
                (weight, weight, 0.0)
            }
            (None, Some(node)) => {
                let weight = self.weight(node);
                (weight, 0.0, weight)
            }
            (Some(one), Some(two)) => {
                let children_one = one.children().unwrap_or(&[]);
                let children_two = two.children().unwrap_or(&[]);

                let (mut cost, mut weight_one, mut weight_two) = (0.0, 0.0, 0.0);
                for i in 0..children_one.len().max(children_two.len()) {
                    let (c, w1, w2) = self.walk(children_one.get(i), children_two.get(i));
                    cost += c;
                    weight_one += w1;
                    weight_two += w2;
                }

                let here = if one.value() == two.value() { 0.0 } else { 2.0 };
                (
                    here + cost / self.k,
                    1.0 + weight_one / self.k,
                    1.0 + weight_two / self.k,
                )
            }
        }
    }

    /// Total weight of a subtree: 1 for its root plus `1 / k` of its children's weights.
    fn weight<T>(&self, node: &TreeNode<T>) -> f32 {
        1.0 + node
            .children()
            .unwrap_or(&[])
            .iter()
            .map(|child| self.weight(child))
            .sum::<f32>()
            / self.k
    }
}

impl Default for TreeDistance {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialEq> Distance<TreeNode<T>> for TreeDistance {
    fn calculate(&self, one: &TreeNode<T>, two: &TreeNode<T>) -> f32 {
        self.between(one, two)
    }
}

impl<T> Distance<Phenotype<TreeChromosome<T>>> for TreeDistance
where
    T: Clone + PartialEq,
{
    fn calculate(
        &self,
        one: &Phenotype<TreeChromosome<T>>,
        two: &Phenotype<TreeChromosome<T>>,
    ) -> f32 {
        let trees_one = one.genotype().iter().map(|c| c.as_ref());
        let trees_two = two.genotype().iter().map(|c| c.as_ref());
        let (mut trees_one, mut trees_two) = (trees_one.peekable(), trees_two.peekable());

        let mut total = 0.0;
        let mut count = 0;
        while trees_one.peek().is_some() || trees_two.peek().is_some() {
            total += match (trees_one.next(), trees_two.next()) {
                (Some(a), Some(b)) => self.between(a, b),
                _ => 1.0,
            };
            count += 1;
        }

        if count > 0 { total / count as f32 } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeExt;
    use crate::{NodeType, Op, Tree};
    use radiate_core::random_provider;

    fn node(op: Op<f32>, children: Vec<TreeNode<Op<f32>>>) -> TreeNode<Op<f32>> {
        children
            .into_iter()
            .fold(TreeNode::from(op), |parent, child| parent.attach(child))
    }

    fn leaf(op: Op<f32>) -> TreeNode<Op<f32>> {
        TreeNode::from(op)
    }

    /// add(mul(x0, x1), x2)
    fn sample() -> TreeNode<Op<f32>> {
        node(
            Op::add(),
            vec![
                node(Op::mul(), vec![leaf(Op::var(0)), leaf(Op::var(1))]),
                leaf(Op::var(2)),
            ],
        )
    }

    #[test]
    fn test_identical_trees_are_zero() {
        assert_eq!(TreeDistance::new().between(&sample(), &sample()), 0.0);
    }

    #[test]
    fn test_same_shape_all_different_is_one() {
        let other = node(
            Op::sub(),
            vec![
                node(Op::div(), vec![leaf(Op::var(3)), leaf(Op::var(4))]),
                leaf(Op::var(5)),
            ],
        );

        assert!((TreeDistance::new().between(&sample(), &other) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_root_change_outweighs_leaf_change() {
        let distance = TreeDistance::new();

        let mut root_changed = sample();
        root_changed.set_value(Op::sub());

        let mut leaf_changed = sample();
        leaf_changed.get_mut(2).unwrap().set_value(Op::var(9)); // x0, two levels down

        let root = distance.between(&sample(), &root_changed);
        let leaf = distance.between(&sample(), &leaf_changed);
        assert!(root > leaf && leaf > 0.0, "root {root}, leaf {leaf}");

        // With k = 1 every node counts the same, so the two changes are equal.
        let flat = TreeDistance::new().with_k(1.0);
        assert_eq!(
            flat.between(&sample(), &root_changed),
            flat.between(&sample(), &leaf_changed)
        );
    }

    #[test]
    fn test_missing_subtrees_count_and_grow_with_size() {
        let distance = TreeDistance::new();

        // add(mul(x0, x1), x2) vs add(mul(x0, x1), x2, <extra>)
        let with_extra = |extra: TreeNode<Op<f32>>| {
            let mut tree = sample();
            tree.add_child(extra);
            tree
        };

        let small = distance.between(&sample(), &with_extra(leaf(Op::var(0))));
        let large = distance.between(
            &sample(),
            &with_extra(node(Op::mul(), vec![leaf(Op::var(0)), leaf(Op::var(1))])),
        );

        assert!(small > 0.0 && large > small, "small {small}, large {large}");
    }

    #[test]
    fn test_random_trees_are_symmetric_and_in_range() {
        random_provider::scoped_seed(17, || {
            let store = vec![
                (
                    NodeType::Vertex,
                    vec![Op::<f32>::add(), Op::mul(), Op::sum(), Op::log()],
                ),
                (NodeType::Leaf, vec![Op::var(0), Op::var(1), Op::var(2)]),
            ];
            let trees = (0..40)
                .map(|i| Tree::with_depth(1 + i % 5, store.clone()))
                .collect::<Vec<_>>();

            let distance = TreeDistance::new();
            for a in &trees {
                for b in &trees {
                    let (a, b) = (a.root().unwrap(), b.root().unwrap());
                    let ab = distance.between(a, b);
                    assert!((0.0..=1.0).contains(&ab), "{ab}");
                    assert_eq!(ab, distance.between(b, a));
                }
                assert_eq!(distance.between(a.root().unwrap(), a.root().unwrap()), 0.0);
            }
        });
    }

    #[test]
    fn test_phenotypes_average_over_trees() {
        let phenotype = |trees: Vec<TreeNode<Op<f32>>>| {
            let chromosomes = trees
                .into_iter()
                .map(|tree| TreeChromosome::from(tree))
                .collect::<Vec<_>>();
            Phenotype::from((chromosomes, 0))
        };

        let mut changed = sample();
        changed.set_value(Op::sub());
        let one_tree = TreeDistance::new().between(&sample(), &changed);

        // Two trees, one identical and one changed: half the single-tree distance.
        let a = phenotype(vec![sample(), sample()]);
        let b = phenotype(vec![sample(), changed]);
        let both = TreeDistance::new().calculate(&a, &b);
        assert!((both - one_tree / 2.0).abs() < 1e-6);

        // A tree with no counterpart counts as 1.
        let c = phenotype(vec![sample()]);
        let d = phenotype(vec![sample(), sample()]);
        assert!((TreeDistance::new().calculate(&c, &d) - 0.5).abs() < 1e-6);
    }
}
