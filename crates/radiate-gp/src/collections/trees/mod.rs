mod builder;
mod chromosome;
mod codec;
mod crossover;
mod eval;
mod iter;
mod mutator;
mod node;
mod tree;

pub use chromosome::TreeChromosome;
pub use codec::TreeCodec;
pub use crossover::TreeCrossover;
pub use iter::TreeIterator;
pub use mutator::HoistMutator;
pub use node::TreeNode;
pub use tree::Tree;

// use std::collections::HashMap;
// use radiate_core::{Chromosome, Genotype, Population};
// use radiate_gp::{NodeType, Tree, TreeChromosome, TreeNode, collections::NodeStore};
// use crate::traits::{ModelLearner, ProbabilisticModel};

// #[derive(Clone)]
// pub struct PcfgCfg {
//     pub alpha: f64,        // smoothing
//     pub max_depth: usize,  // depth guard
//     pub max_size: usize,   // size guard
//     pub prefer_leaf_after: usize, // bias leaf as depth grows
// }

// impl Default for PcfgCfg {
//     fn default() -> Self {
//         Self { alpha: 0.5, max_depth: 12, max_size: 64, prefer_leaf_after: 6 }
//     }
// }

// // Context key: (node_type, parent_op_name) – start simple; extend later if needed
// #[derive(Hash, Eq, PartialEq, Clone)]
// struct Ctx(&'static str, &'static str);

// fn nt_str(nt: NodeType) -> &'static str {
//     match nt {
//         NodeType::Root => "Root",
//         NodeType::Vertex => "Vertex",
//         NodeType::Leaf => "Leaf",
//         _ => "Other",
//     }
// }

// pub struct TreePcfgModel<T: Clone + PartialEq> {
//     pub cfg: PcfgCfg,
//     pub store: Option<NodeStore<T>>,
//     // P(op_name | ctx) as normalized probabilities
//     pub p_op_given_ctx: HashMap<Ctx, Vec<(&'static str, f64)>>,
//     // For variable-arity ops: P(k | op_name)
//     pub p_k_given_op: HashMap<&'static str, Vec<(usize, f64)>>,
// }

// pub struct TreePcfgLearner<T: Clone + PartialEq> {
//     pub cfg: PcfgCfg,
// }

// impl<T: Clone + PartialEq + 'static> ModelLearner<TreeChromosome<T>> for TreePcfgLearner<T> {
//     type Model = TreePcfgModel<T>;

//     fn learn(&self, parents: &Population<TreeChromosome<T>>) -> Self::Model {
//         // Count tables
//         let mut counts_ctx_op: HashMap<Ctx, HashMap<&'static str, f64>> = HashMap::new();
//         let mut counts_op_k: HashMap<&'static str, HashMap<usize, f64>> = HashMap::new();
//         let store = parents[0].genotype()[0].get_store();

//         // Traverse each parent tree and update counts
//         for ind in parents.iter() {
//             // Each genotype may carry multiple trees; we assume single-root by convention
//             let chr = &ind.genotype()[0];
//             let root = chr.root();
//             visit(None, NodeType::Root, root, &mut counts_ctx_op, &mut counts_op_k);
//         }

//         // Smooth + normalize P(op | ctx)
//         let mut p_op_given_ctx = HashMap::new();
//         for (ctx, cmap) in counts_ctx_op.into_iter() {
//             let mut entries: Vec<(&'static str, f64)> = cmap.into_iter().collect();
//             let s: f64 = entries.iter().map(|(_, v)| v + self.cfg.alpha).sum();
//             for (_, v) in entries.iter_mut() { *v = (*v + self.cfg.alpha) / s; }
//             entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
//             p_op_given_ctx.insert(ctx, entries);
//         }

//         // Smooth + normalize P(k | op)
//         let mut p_k_given_op = HashMap::new();
//         for (op, kmap) in counts_op_k.into_iter() {
//             let mut entries: Vec<(usize, f64)> = kmap.into_iter().collect();
//             let s: f64 = entries.iter().map(|(_, v)| v + self.cfg.alpha).sum();
//             for (_, v) in entries.iter_mut() { *v = (*v + self.cfg.alpha) / s; }
//             entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
//             p_k_given_op.insert(op, entries);
//         }

//         TreePcfgModel { cfg: self.cfg.clone(), store, p_op_given_ctx, p_k_given_op }
//     }
// }

// // Helper: increment counts over a tree
// fn visit<T: Clone + PartialEq>(
//     parent_name: Option<&'static str>,
//     node_type: NodeType,
//     node: &TreeNode<T>,
//     counts_ctx_op: &mut HashMap<Ctx, HashMap<&'static str, f64>>,
//     counts_op_k: &mut HashMap<&'static str, HashMap<usize, f64>>,
// ) {
//     let op_name = node.value().name(); // requires Op<T> to expose stable name(); for custom T, adapt
//     let ctx = Ctx(nt_str(node_type), parent_name.unwrap_or("<none>"));
//     *counts_ctx_op.entry(ctx).or_default().entry(op_name).or_default() += 1.0;

//     let k = node.children().map(|c| c.len()).unwrap_or(0);
//     *counts_op_k.entry(op_name).or_default().entry(k).or_default() += 1.0;

//     if let Some(children) = node.children() {
//         for child in children {
//             visit(Some(op_name), NodeType::Vertex, child, counts_ctx_op, counts_op_k);
//         }
//     } else {
//         // Leaf – nothing further
//     }
// }

// // Minimal trait needed for naming; for Op<f32> we already have static names
// trait Named {
//     fn name(&self) -> &'static str;
// }

// // Op<f32> implements names like "add","mul",...; implement Named for it
// impl Named for radiate_gp::ops::operation::Op<f32> {
//     fn name(&self) -> &'static str {
//         match self { Self::Fn(n, _, _) => n, Self::Var(n, _) => n, Self::Const(n, _) => n, Self::MutableConst{ name, .. } => name, Self::Value(n, ..) => n }
//     }
// }

// impl ProbabilisticModel<TreeChromosome<radiate_gp::ops::operation::Op<f32>>> for TreePcfgModel<radiate_gp::ops::operation::Op<f32>> {
//     fn name(&self) -> &'static str { "PCFG-Tree" }

//     fn sample(&self, n: usize) -> Vec<Genotype<TreeChromosome<radiate_gp::ops::operation::Op<f32>>>> {
//         (0..n).map(|_| {
//             let mut size = 0usize;
//             let root = self.sample_node("<none>", NodeType::Root, 0, &mut size);
//             Genotype::from(TreeChromosome::new(vec![root], self.store.clone(), None))
//         }).collect()
//     }
// }

// impl TreePcfgModel<radiate_gp::ops::operation::Op<f32>> {
//     fn sample_node(
//         &self,
//         parent_name: &'static str,
//         node_type: NodeType,
//         depth: usize,
//         size: &mut usize,
//     ) -> TreeNode<radiate_gp::ops::operation::Op<f32>> {
//         *size += 1;
//         let depth_bias_leaf = depth >= self.cfg.prefer_leaf_after;
//         // choose op from P(op|ctx), fallback to store if unseen
//         let ctx = Ctx(nt_str(node_type), parent_name);
//         let op = choose_op(&self.p_op_given_ctx, &self.store, &ctx, node_type)
//             .unwrap_or_else(|| default_op(&self.store, node_type));

//         // determine k
//         let k = arity_for(&op, &self.p_k_given_op, depth, self.cfg.max_depth, depth_bias_leaf);

//         if k == 0 || depth + 1 >= self.cfg.max_depth || *size >= self.cfg.max_size {
//             TreeNode::new(op)
//         } else {
//             let mut children = Vec::with_capacity(k);
//             for _ in 0..k {
//                 let child = self.sample_node(op.name(), NodeType::Vertex, depth + 1, size);
//                 children.push(child);
//             }
//             TreeNode::with_children(op, children)
//         }
//     }
// }

// // Utility selection helpers – left abstract here; pick top-probable or sample multinomial
// fn choose_op<T: Clone + PartialEq>(
//     p_op_given_ctx: &HashMap<Ctx, Vec<(&'static str, f64)>>,
//     store: &Option<NodeStore<T>>,
//     ctx: &Ctx,
//     node_type: NodeType,
// ) -> Option<T> {
//     // For Op<f32>, we can map name -> actual Op via store or a registry you control.
//     // Minimal viable: take the most common op name seen for ctx, then find a matching value in store for this node type.
//     if let Some(entries) = p_op_given_ctx.get(ctx) {
//         if let Some((name, _)) = entries.first() {
//             if let Some(store) = store {
//                 if let Some(vs) = store.map_by_type(node_type, |vals| vals.iter().map(|v| v.value().clone()).collect()) {
//                     // naive linear find by name; for Op<f32> names are embedded
//                     for v in vs {
//                         if v.name() == *name { return Some(v); }
//                     }
//                 }
//             }
//         }
//     }
//     None
// }

// fn default_op<T: Clone + PartialEq>(store: &Option<NodeStore<T>>, node_type: NodeType) -> T {
//     store.as_ref()
//         .and_then(|s| s.map_by_type(node_type, |vals| vals.first().map(|v| v.value().clone())))
//         .flatten()
//         .expect("Store must provide at least one op for node_type")
// }

// fn arity_for(
//     op: &radiate_gp::ops::operation::Op<f32>,
//     p_k_given_op: &HashMap<&'static str, Vec<(usize, f64)>>,
//     depth: usize,
//     max_depth: usize,
//     depth_bias_leaf: bool,
// ) -> usize {
//     use radiate_gp::Arity;
//     match op {
//         radiate_gp::ops::operation::Op::Fn(_, ar, _) => match *ar {
//             Arity::Zero => 0,
//             Arity::Exact(k) => k,
//             Arity::Any => {
//                 let name = op.name();
//                 let k = p_k_given_op.get(name).and_then(|v| v.first().map(|(k, _)| *k)).unwrap_or(2);
//                 if depth + 1 >= max_depth { 0 } else if depth_bias_leaf { k.min(2) } else { k }
//             }
//         },
//         radiate_gp::ops::operation::Op::Var(_, _) => 0,
//         radiate_gp::ops::operation::Op::Const(_, _) => 0,
//         radiate_gp::ops::operation::Op::MutableConst{ .. } => 1,
//         radiate_gp::ops::operation::Op::Value(_, ar, _, _) => match *ar {
//             Arity::Zero => 0, Arity::Exact(k) => k, Arity::Any => { if depth + 1 >= max_depth { 0 } else { 2 } }
//         },
//     }
// }
