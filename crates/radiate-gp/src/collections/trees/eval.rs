use super::Tree;
use crate::{Eval, EvalMut, TreeNode, eval::EvalInto, node::Node};
use radiate_utils::Matrix;
use smallvec::SmallVec;

/// Input columns are collected into a stack array up to this fan-in, so the common case doesn't
/// allocate per node; wider nodes spill to the heap.
const STACK_INPUTS: usize = 8;

/// Reusable buffers for evaluating trees a whole column (every sample) at a time.
///
/// A tree needs no compiled plan: it has no cycles, so its own post-order *is* the schedule.
/// The cache is just a stack of columns, each `samples` long. Evaluating a node reserves its
/// output column, lets each child push its column above it, runs the node's column [EvalInto]
/// over those, then pops the children:
///
/// ```text
///   add(mul(x0, x1), x2)                 stack (one column per slot)
///
///   push add      reserve out            [ add ]
///     push mul    reserve out            [ add | mul ]
///       x0, x1    leaves read features   [ add | mul | x0 | x1 ]
///     mul ← x0*x1, pop children          [ add | mul ]
///     x2                                 [ add | mul | x2 ]
///   add ← mul+x2, pop children           [ add ]
/// ```
///
/// The stack only ever holds one path's worth of columns (about height × arity), not one per
/// node, and keeps its allocation between trees.
#[derive(Default, Clone)]
pub struct TreeEvalCache<V> {
    columns: Vec<V>,
}

impl<V> TreeEvalCache<V>
where
    V: Copy + Default,
{
    /// Evaluates each root over every sample. `features` is `features × samples` (each row one
    /// feature's column) and `outputs` receives `roots × samples`: root `j`'s value for sample
    /// `r` is `outputs[(j, r)]`. Bit-identical to evaluating each sample one at a time.
    pub fn eval_columns<T>(
        &mut self,
        roots: &[&TreeNode<T>],
        features: &Matrix<V>,
        outputs: &mut Matrix<V>,
    ) where
        T: for<'c> EvalInto<[&'c [V]], [V]>,
    {
        let n = features.cols();
        outputs.reshape_and_fill(roots.len(), n, V::default());
        if n == 0 {
            return;
        }

        // Leaves read the sample's features directly
        let feature_columns = (0..features.rows())
            .map(|i| features.row(i))
            .collect::<SmallVec<[&[V]; STACK_INPUTS]>>();

        for (j, root) in roots.iter().enumerate() {
            self.columns.clear();
            push_column(root, &feature_columns, n, &mut self.columns);
            outputs.row_mut(j).copy_from_slice(&self.columns[..n]);
        }
    }
}

/// Pushes `node`'s column (`n` values) onto `stack`. Mirrors `Eval for &TreeNode`: a node
/// without children evaluates the features, otherwise it evaluates its children's values.
fn push_column<T, V>(node: &TreeNode<T>, features: &[&[V]], n: usize, stack: &mut Vec<V>)
where
    T: for<'c> EvalInto<[&'c [V]], [V]>,
    V: Copy + Default,
{
    let base = stack.len();
    stack.resize(base + n, V::default());

    let Some(children) = node.children() else {
        node.value().eval_into(features, &mut stack[base..]);
        return;
    };

    for child in children {
        push_column(child, features, n, stack);
    }

    {
        let (below, above) = stack.split_at_mut(base + n);
        let inputs = above
            .chunks_exact(n)
            .collect::<SmallVec<[&[V]; STACK_INPUTS]>>();
        node.value().eval_into(&inputs, &mut below[base..]);
    }

    stack.truncate(base + n);
}

impl<T, V> Eval<[V], V> for Tree<T>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval(&self, input: &[V]) -> V {
        self.root()
            .map(|root| root.eval(input))
            .unwrap_or_else(|| panic!("Tree has no root node."))
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for Tree<T>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        if let Some(root) = self.root() {
            vec![root.eval(input)]
        } else {
            vec![]
        }
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for Vec<Tree<T>>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        self.iter_mut()
            .filter_map(|tree| tree.root().map(|root| root.eval(input)))
            .collect::<Vec<V>>()
    }
}

impl<T, V> Eval<[V], V> for TreeNode<T>
where
    T: Eval<[V], V>,
    V: Clone,
{
    #[inline]
    fn eval(&self, input: &[V]) -> V {
        if let Some(children) = self.children() {
            let inputs = children
                .iter()
                .map(|child| child.eval(input))
                .collect::<SmallVec<[V; STACK_INPUTS]>>();

            return self.value().eval(&inputs);
        }

        self.value().eval(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeType, Op, TreeNode, all_ops};
    use radiate_core::random_provider;

    /// Evaluates `roots` one sample at a time and as a batch, asserting the two agree bit for bit.
    fn assert_rows_match_batch(roots: &[&TreeNode<Op<f32>>], samples: &[Vec<f32>]) {
        let width = samples.first().map_or(0, |s| s.len());
        let mut features = Matrix::default();

        features.reshape_and_fill(width, samples.len(), 0.0);
        for (r, sample) in samples.iter().enumerate() {
            for (f, &x) in sample.iter().enumerate() {
                features[(f, r)] = x;
            }
        }

        let mut batch = Matrix::default();
        TreeEvalCache::default().eval_columns(roots, &features, &mut batch);

        assert_eq!(batch.rows(), roots.len());
        assert_eq!(batch.cols(), samples.len());

        for (r, sample) in samples.iter().enumerate() {
            for (j, root) in roots.iter().enumerate() {
                let value = root.eval(sample);
                assert_eq!(
                    value.to_bits(),
                    batch[(j, r)].to_bits(),
                    "sample {r}, root {j}: row {value} vs batch {}",
                    batch[(j, r)]
                );
            }
        }
    }

    fn samples(width: usize, n: usize) -> Vec<Vec<f32>> {
        (0..n)
            .map(|r| {
                (0..width)
                    .map(|f| ((r * 7 + f * 3) % 11) as f32 * 0.37 - 1.5)
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_random_trees_rows_match_batch() {
        random_provider::seed(42);

        let store = vec![
            (NodeType::Vertex, all_ops::<f32>()),
            (
                NodeType::Leaf,
                (0..3)
                    .map(Op::var)
                    .chain([Op::constant("half", 0.5), Op::constant("minus_two", -2.0)])
                    .collect(),
            ),
        ];

        let trees = (0..200)
            .map(|i| Tree::with_depth(1 + i % 6, store.clone()))
            .collect::<Vec<_>>();

        let data = samples(3, 37);
        for tree in &trees {
            assert_rows_match_batch(&[tree.root().unwrap()], &data);
        }

        // Several roots at once, like a multi-output genotype, sharing one cache.
        let roots = trees.iter().map(|t| t.root().unwrap()).collect::<Vec<_>>();
        assert_rows_match_batch(&roots, &data);
    }

    #[test]
    fn test_wide_node_spills_past_stack_inputs() {
        let wide = (0..STACK_INPUTS + 4).fold(TreeNode::new(Op::sum()), |node, i| {
            node.attach(TreeNode::new(Op::var(i % 2)))
        });
        let root = TreeNode::new(Op::sigmoid()).attach(wide);

        assert_rows_match_batch(&[&root], &samples(2, 10));
    }

    #[test]
    fn test_cache_reused_across_trees_and_sizes() {
        let deep = TreeNode::new(Op::add())
            .attach(
                TreeNode::new(Op::mul())
                    .attach(Op::var(0))
                    .attach(Op::var(1)),
            )
            .attach(TreeNode::new(Op::var(1)));
        let leaf = TreeNode::new(Op::var(0));

        let mut cache = TreeEvalCache::default();
        let mut out = Matrix::default();

        let mut features = Matrix::default();
        features.reshape_and_fill(2, 3, 0.0);
        for r in 0..3 {
            features[(0, r)] = r as f32;
            features[(1, r)] = 2.0;
        }

        cache.eval_columns(&[&deep], &features, &mut out);
        assert_eq!(out.row(0), &[2.0, 4.0, 6.0]);

        cache.eval_columns(&[&leaf, &deep], &features, &mut out);
        assert_eq!(out.row(0), &[0.0, 1.0, 2.0]);
        assert_eq!(out.row(1), &[2.0, 4.0, 6.0]);

        // No samples: an empty `roots × 0` result.
        let mut empty = Matrix::default();
        empty.reshape_and_fill(2, 0, 0.0);
        cache.eval_columns(&[&deep], &empty, &mut out);
        assert_eq!((out.rows(), out.cols()), (1, 0));
    }

    #[test]
    fn test_tree_reduce_simple() {
        let mut root = TreeNode::new(Op::add());

        root.add_child(TreeNode::new(Op::constant("one", 1.0)));
        root.add_child(TreeNode::new(Op::constant("two", 2.0)));

        let result = root.eval(&[]);

        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_tree_reduce_complex() {
        let node = TreeNode::new(Op::add())
            .attach(
                TreeNode::new(Op::mul())
                    .attach(TreeNode::new(Op::constant("two", 2.0)))
                    .attach(TreeNode::new(Op::constant("three", 3.0))),
            )
            .attach(
                TreeNode::new(Op::add())
                    .attach(TreeNode::new(Op::constant("two", 2.0)))
                    .attach(TreeNode::new(Op::var(0))),
            );

        let nine = node.eval(&[1_f32]);
        let ten = node.eval(&[2_f32]);
        let eleven = node.eval(&[3_f32]);

        assert_eq!(nine, 9.0);
        assert_eq!(ten, 10.0);
        assert_eq!(eleven, 11.0);
    }
}
