// pub use factor::{FactorSpec, Potential};
// use value::PgmValue;
// use variable::{VARVAL, Variable};

// use core::num;
// use radiate_core::{Chromosome, Codec, Gene, Genotype, Valid, random_provider};
// use radiate_gp::{
//     Domain, Eval, Factory, Graph, GraphAggregate, GraphNode, Node, NodeBuilder, NodeStore,
//     NodeValue, Op,
//     collections::{GraphChromosome, NodeType},
// };
// use radiate_utils::SortedBuffer;
// use std::{collections::BTreeSet, slice::SliceIndex};

// pub struct ProbDataset {
//     pub observations: Vec<Vec<Option<VARVAL>>>, // length == num_vars per row
//     pub num_vars: usize,
// }

// impl ProbDataset {
//     pub fn new(observations: Vec<Vec<Option<impl Into<VARVAL>>>>) -> Self {
//         let obs = observations
//             .into_iter()
//             .map(|row| row.into_iter().map(|v| v.map(|val| val.into())).collect())
//             .collect::<Vec<Vec<Option<VARVAL>>>>();
//         let num_vars = obs.first().map(|r| r.len()).unwrap_or(0);
//         assert!(
//             obs.iter().all(|r| r.len() == num_vars),
//             "All rows must have same width"
//         );
//         Self {
//             observations: obs,
//             num_vars,
//         }
//     }

//     pub fn rows(&self) -> usize {
//         self.observations.len()
//     }
// }

// /// A factor graph representing a probabilistic graphical model.
// #[derive(Clone, Debug, PartialEq)]
// pub struct FactorGraph {
//     pub variables: Vec<Op<f32>>,
//     pub factors: Vec<(SortedBuffer<usize>, FactorSpec)>,
// }

// // Discrete-only: exact variable elimination / sum-product.
// // Pure Gaussian (all Real with Gaussian/LinearGaussian): exact Gaussian elimination.
// // Conditional Linear Gaussian (CLG) hybrid: exact if the BN meets CLG constraints (continuous nodes have linear-Gaussian CPDs; discrete nodes do not have continuous parents). Use per-discrete-parent-component Gaussians (MixtureByDiscrete).
// // General hybrid: approximate (Gibbs/Metropolis, loopy BP with continuous messages, or EP).
// // API stays the same; backend chooses by inspecting domains/potentials:

// pub trait Infer {
//     fn log_likelihood(&self, model: &FactorGraph, data: &ProbDataset) -> f32;
//     // optional:
//     // fn marginal(&self, var: usize, evidence: &[Option<Value>]) -> Distribution;
// }

// #[derive(Clone, Debug)]
// pub struct PgmCodecConfig {
//     // Explicit variables (ids must be 0..N-1 in order)
//     pub variables: Vec<Op<f32>>,

//     // How many factors to initialize
//     pub num_factors: usize,

//     // Max number of variables allowed in any factor scope
//     pub max_scope: usize,

//     // Enable hybrid potentials
//     pub allow_linear_gaussian: bool, // Real child with Real parents
//     pub allow_mixture_by_discrete: bool, // Real child with Discrete parents
// }

// pub struct PgmCodec {
//     cfg: PgmCodecConfig,
//     node_store: NodeStore<PgmValue>,
// }

// impl PgmCodec {
//     pub fn new(variables: Vec<impl Into<Op<f32>>>, num_factors: usize, max_scope: usize) -> Self {
//         let variables = variables.into_iter().map(|v| v.into()).collect::<Vec<_>>();
//         let cfg = PgmCodecConfig {
//             variables: variables.clone(),
//             num_factors,
//             max_scope,
//             allow_linear_gaussian: true,
//             allow_mixture_by_discrete: true,
//         };
//         Self {
//             cfg,
//             node_store: Self::create_node_store(variables, num_factors, max_scope),
//         }
//     }

//     fn create_node_store(
//         variables: Vec<impl Into<Op<f32>>>,
//         num_factors: usize,
//         max_scope: usize,
//     ) -> NodeStore<PgmValue> {
//         let temp = vec![(
//             NodeType::Input,
//             variables
//                 .into_iter()
//                 .enumerate()
//                 .map(|(i, v)| {
//                     let v = v.into();
//                     PgmValue::Variable(v)
//                 })
//                 .collect::<Vec<_>>(),
//         )];

//         NodeStore::from(temp)
//     }

//     fn card_of(&self, var_id: usize) -> Option<usize> {
//         match self.cfg.variables[var_id].domain() {
//             Some(Domain::Categorical(k)) => Some(*k),
//             _ => None,
//         }
//     }

//     fn is_discrete_scope(&self, scope: &[usize]) -> bool {
//         scope
//             .iter()
//             .all(|&v| matches!(self.cfg.variables[v].domain(), Some(Domain::Categorical(_))))
//     }

//     fn pick_linear_gaussian(&self, scope: &[usize]) -> Option<(usize, Vec<usize>)> {
//         // Choose one Real child and Real parents (>=0)
//         let real_vars: Vec<usize> = scope
//             .iter()
//             .copied()
//             .filter(|&v| matches!(self.cfg.variables[v].domain(), Some(Domain::Value)))
//             .collect::<Vec<usize>>();

//         if real_vars.is_empty() {
//             return None;
//         }

//         // Child = first real; parents = remaining real in scope (could be empty)
//         let child = real_vars[0];
//         let parents = real_vars.iter().copied().filter(|&v| v != child).collect();

//         Some((child, parents))
//     }

//     fn pick_mixture_by_discrete(&self, scope: &[usize]) -> Option<usize> {
//         // Exactly one Real child and >=1 Discrete parents
//         let real_vars: Vec<usize> = scope
//             .iter()
//             .copied()
//             .filter(|&v| matches!(self.cfg.variables[v].domain(), Some(Domain::Value)))
//             .collect();

//         if real_vars.len() != 1 {
//             return None;
//         }

//         let has_discrete_parent = scope
//             .iter()
//             .any(|&v| matches!(self.cfg.variables[v].domain(), Some(Domain::Categorical(_))));

//         if !has_discrete_parent {
//             return None;
//         }

//         Some(real_vars[0]) // child
//     }

//     fn random_scope(&self) -> SortedBuffer<usize> {
//         let num_vars = self.cfg.variables.len();
//         let max_k = self.cfg.max_scope.min(num_vars).max(1);
//         let k = random_provider::range(1..max_k + 1);

//         let mut s = BTreeSet::new();
//         while s.len() < k {
//             s.insert(random_provider::range(0..num_vars));
//         }

//         s.into_iter().collect()
//     }

//     fn random_shape(&self) -> Vec<usize> {
//         self.random_scope()
//             .iter()
//             .map(|&v| self.card_of(v).unwrap_or(1))
//             .collect()
//     }

//     fn init_discrete_table(&self, scope: &[usize]) -> Potential {
//         // Uniform log-table
//         let size = scope.iter().fold(1, |acc, &v| {
//             acc * self.card_of(v).expect("Expected Discrete domain")
//         });

//         let val = -((size as f64).ln()) as f32;
//         Potential::DiscreteTable(vec![val; size])
//     }

//     fn init_gaussian(&self) -> Potential {
//         let mean = random_provider::range(-1.0..1.0) as f32;
//         let var = random_provider::range(0.1..1.0) as f32; // keep positive

//         Potential::Gaussian(mean, var)
//     }

//     fn init_linear_gaussian(&self, parents: usize) -> Potential {
//         let weights = (0..parents)
//             .map(|_| random_provider::range(-1.0..1.0) as f64)
//             .collect::<Vec<_>>();

//         let bias = random_provider::range(-0.5..0.5) as f64;
//         let var = random_provider::range(0.05..0.5) as f64;

//         Potential::LinearGaussian { weights, bias, var }
//     }

//     fn init_mixture_by_discrete(&self, scope: &[usize], child: usize) -> Potential {
//         // Components per discrete parent assignment
//         let parents = scope
//             .iter()
//             .copied()
//             .filter(|&v| v != child)
//             .collect::<Vec<_>>();

//         let comps = parents.iter().fold(1, |acc, &v| {
//             acc * self.card_of(v).expect("Mixture requires discrete parents")
//         });

//         let mean = (0..comps)
//             .map(|_| random_provider::range(-1.0..1.0) as f64)
//             .collect::<Vec<_>>();
//         let var = (0..comps)
//             .map(|_| random_provider::range(0.05..0.5) as f64)
//             .collect::<Vec<_>>();

//         Potential::MixtureByDiscrete { mean, var }
//     }

//     fn make_factor_potential(&self, scope: &[usize]) -> Potential {
//         // Priority: exact families when compatible; otherwise fall back to discrete/gaussian
//         if scope.len() == 1 {
//             // Unary: prior
//             let v = scope[0];
//             return match self.cfg.variables[v].domain() {
//                 Some(Domain::Categorical(_)) => self.init_discrete_table(scope),
//                 Some(Domain::Value) => self.init_gaussian(),
//                 _ => panic!("Unexpected domain for unary factor"),
//             };
//         }

//         // Multi-var scopes
//         if self.is_discrete_scope(scope) {
//             return self.init_discrete_table(scope);
//         }

//         // Hybrid possibilities
//         if self.cfg.allow_mixture_by_discrete {
//             if let Some(child) = self.pick_mixture_by_discrete(scope) {
//                 return self.init_mixture_by_discrete(scope, child);
//             }
//         }
//         if self.cfg.allow_linear_gaussian {
//             if let Some((_child, parents_real)) = self.pick_linear_gaussian(scope) {
//                 return self.init_linear_gaussian(parents_real.len());
//             }
//         }

//         // Fallbacks:
//         // - If there is any Real and any Discrete but neither hybrid enabled, choose a unary prior
//         //   for a random variable as a simple starting factor (keeps model valid).
//         if scope
//             .iter()
//             .any(|&v| matches!(self.cfg.variables[v].domain(), Some(Domain::Value)))
//             && scope
//                 .iter()
//                 .any(|&v| matches!(self.cfg.variables[v].domain(), Some(Domain::Categorical(_))))
//         {
//             let pick = scope[random_provider::range(0..scope.len())];
//             return match self.cfg.variables[pick].domain() {
//                 Some(Domain::Categorical(_)) => self.init_discrete_table(&[pick]),
//                 Some(Domain::Value) => self.init_gaussian(),
//                 _ => panic!("Unexpected domain for unary factor"),
//             };
//         }

//         // Otherwise default to a Gaussian-like unary for a Real-only scope
//         self.init_gaussian()
//     }
// }

// impl Codec<GraphChromosome<PgmValue>, FactorGraph> for PgmCodec {
//     fn encode(&self) -> Genotype<GraphChromosome<PgmValue>> {
//         let num_variables = self.cfg.variables.len();
//         let num_factors = self.cfg.num_factors;

//         let temp_store = NodeStore::from(vec![
//             (
//                 NodeType::Input,
//                 (0..num_variables)
//                     .map(|v| {
//                         let var_op = self.cfg.variables[v].clone();
//                         var_op.into()
//                     })
//                     .collect::<Vec<_>>(),
//             ),
//             (
//                 NodeType::Vertex,
//                 vec![
//                     // Op::nd_array(self.random_shape()),
//                     Op::nd_array(self.random_shape()),
//                     Op::nd_array(self.random_shape()),
//                 ],
//             ),
//             (NodeType::Output, vec![Op::sum()]),
//         ]);

//         println!("Temp store: {:?}", temp_store);

//         let builder = NodeBuilder::new(temp_store.clone());
//         let mut agg = GraphAggregate::new();

//         let inputs = builder.input(self.cfg.variables.len());

//         let vertices = builder.vertices(num_factors);

//         agg = agg.insert(&inputs);

//         let outputs = builder.output(1);

//         // agg = agg.many_to_one(&inputs, &vertices);
//         agg = agg.all_to_all(&inputs, &vertices);
//         agg = agg.all_to_all(&vertices, &outputs);
//         let graph = agg.build();

//         println!("Initial graph: {:?}", graph);

//         // 1) Sample factor scopes
//         let mut scopes = Vec::with_capacity(num_factors);
//         for _ in 0..num_factors {
//             scopes.push(self.random_scope());
//         }

//         let mut factors = Vec::with_capacity(num_factors);
//         for scope in scopes.iter() {
//             let potential = self.make_factor_potential(&scope);
//             let temp = Op::nd_array(
//                 scope
//                     .iter()
//                     .map(|&v| self.card_of(v).unwrap_or(1))
//                     .collect::<Vec<_>>(),
//             );
//             // println!("Factor scope: {:?}, potential: {:?}", scope, temp);
//             factors.push(FactorSpec { potential });
//         }

//         // 3) Build nodes: variables [0..n), factors [n..n+m)
//         let mut nodes = Vec::with_capacity(num_variables + num_factors);
//         // let mut other = Vec::with_capacity(num_variables + num_factors);

//         // 1) Variable nodes
//         for var_id in 0..num_variables {
//             // list factor indices that include this variable
//             let factor_idxs = scopes
//                 .iter()
//                 .enumerate()
//                 .filter(|(_, sc)| sc.contains(&var_id))
//                 .map(|(j, _)| num_variables + j)
//                 .collect::<SortedBuffer<usize>>();

//             let allele = PgmValue::Variable(self.cfg.variables[var_id].clone());
//             nodes.push(
//                 GraphNode::from((var_id, NodeType::Input, allele))
//                     .with_outgoing(factor_idxs.iter().cloned().into_iter()),
//             );
//         }

//         // 2) Factor nodes
//         for (j, scope) in scopes.iter().enumerate() {
//             let idx = num_variables + j;

//             let allele = PgmValue::Factor(FactorSpec {
//                 potential: factors[j].potential.clone(),
//             });

//             nodes.push(GraphNode::from((
//                 idx,
//                 NodeType::Vertex,
//                 allele,
//                 scope.clone(),
//                 scope.clone(),
//             )));
//         }

//         Genotype::from(GraphChromosome::from(nodes))
//     }

//     fn decode(&self, genotype: &Genotype<GraphChromosome<PgmValue>>) -> FactorGraph {
//         let mut variables = Vec::new();
//         let mut factors = Vec::new();

//         for (idx, gene) in genotype[0].iter().enumerate() {
//             match gene.allele() {
//                 PgmValue::Variable(v) => variables.push(v.clone()),
//                 PgmValue::Factor(f) => {
//                     factors.push((gene.incoming_buffer().clone(), f.clone()));
//                 }
//             }
//         }

//         // variables.sort_by_key(|(idx, _)| *idx);
//         // let variables = variables.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
//         FactorGraph { variables, factors }
//     }
// }

// // helpers
// fn cards(vars: &[Op<f32>]) -> Vec<usize> {
//     vars.iter()
//         .map(|v| match v.domain() {
//             Some(Domain::Categorical(k)) => *k,
//             _ => 1,
//         })
//         .collect()
// }

// // fn logp_assignment(model: &FactorGraph, assign: &[usize]) -> f64 {
// //     let cards = cards(&model.variables);
// //     let mut sum = 0f64;
// //     for (incoming, f) in &model.factors {
// //         match &f.potential {
// //             Potential::DiscreteTable(log_table) => {
// //                 let idx = assignment_index(&incoming, assign, &cards);
// //                 sum += log_table[idx] as f64;
// //             }
// //             _ => return f64::NEG_INFINITY, // not handled in discrete MVP
// //         }
// //     }
// //     sum
// // }

// // fn assignment_index(scope: &[usize], assign: &[usize], cards: &[usize]) -> usize {
// //     // row-major in scope order
// //     let mut idx = 0;
// //     let mut stride = 1;
// //     println!("assignment_index:");
// //     for (pos, &vid) in scope.iter().enumerate() {
// //         if pos > 0 {
// //             stride *= cards[scope[pos - 1]];
// //         }
// //         println!("vid: {}, assign: {}, stride: {}", vid, assign[vid], stride);
// //         idx += assign[vid] * stride;
// //     }
// //     idx
// // }

// fn logp_assign_graph(graph: &Graph<Op<f32>>, assign: &[usize]) -> f32 {
//     let inputs = graph.inputs().map(|n| n.index()).collect::<Vec<_>>();
//     let mut total_logp = 0.0;

//     for node in graph.inputs() {
//         for output in node.outgoing() {
//             total_logp += graph[*output].allele().eval(&[assign[node.index()] as f32]);
//         }
//     }

//     // for node in graph.outputs() {
//     //     let idx = assignment_index(node.incoming(), assign, &inputs);
//     //     total_logp += graph[idx].allele().eval(&[1.0]);
//     // }

//     total_logp
// }

// fn logp_row_graph(graph: &Graph<Op<f32>>, row: &[Option<usize>]) -> f64 {
//     // build full joint over missing by enumeration
//     // let n = graph.inputs().len();
//     // let cards = cards(&graph.input_variables());

//     let inputs = graph
//         .inputs()
//         .enumerate()
//         .map(|(i, v)| (v, row[i]))
//         .collect::<Vec<_>>();

//     // observed as indices
//     // let mut obs = vec![None; n];
//     // for i in 0..n {
//     //     obs[i] = row[i];
//     // }

//     // unknown variable ids

//     // let unknown = (0..n).filter(|&i| obs[i].is_none()).collect::<Vec<_>>();
//     let unknown = inputs
//         .iter()
//         .filter(|(i, v)| v.is_none())
//         .map(|(i, _)| i)
//         .collect::<Vec<_>>();
//     if unknown.is_empty() {
//         let mut full = vec![0; inputs.len()];
//         for i in 0..inputs.len() {
//             full[i] = inputs[i].1.unwrap();
//         }

//         println!("\n\nFull assignment: {:?}", full);

//         return logp_assign_graph(graph, &full) as f64;
//     }

//     // enumerate unknowns
//     let mut log_total = f64::NEG_INFINITY;
//     // multi-cartesian product over unknown states
//     let mut counters = vec![0; unknown.len()];
//     // let dims = unknown.iter().map(|&i| cards[i] as f32).collect::<Vec<_>>();
//     let dims = inputs
//         .iter()
//         .map(|&i| i.0.index() as usize)
//         .collect::<Vec<_>>();

//     loop {
//         let mut full = vec![0; inputs.len()];
//         for i in 0..inputs.len() {
//             full[i] = inputs[i].0.index(); //.unwrap_or(0);
//         }

//         for (k, &vid) in unknown.iter().enumerate() {
//             // full[vid] = counters[k];
//             full[vid.index()] = counters[k];
//         }

//         // let lp = graph.logp_assignment(&full);
//         let lp = logp_assign_graph(graph, &full);
//         log_total = log_sum_exp(log_total, lp as f64);

//         // increment mixed-radix counter
//         let mut carry = true;
//         for k in 0..counters.len() {
//             if !carry {
//                 break;
//             }

//             counters[k] += 1;
//             if counters[k] == dims[k] {
//                 counters[k] = 0;
//                 carry = true;
//             } else {
//                 carry = false;
//             }
//         }

//         if carry {
//             break;
//         }
//     }

//     log_total
// }

// fn assignment_index(scope: &[usize], assign: &[usize], cards: &[usize]) -> usize {
//     // row-major in scope order
//     let mut idx = 0;
//     let mut stride = 1;
//     println!("\nassignment_index:");
//     for (pos, &vid) in scope.iter().enumerate() {
//         if pos > 0 {
//             stride *= cards[scope[pos - 1]];
//         }
//         println!(
//             "assign: {:?}, vid: {}, assign: {}, stride: {}, idx: {}",
//             assign, vid, assign[vid], stride, idx
//         );
//         idx += assign[vid] * stride;
//         println!(" updated idx: {}", idx);
//     }

//     println!("final idx: {}", idx);
//     idx
// }

// fn log_sum_exp(a: f64, b: f64) -> f64 {
//     if a.is_infinite() && a.is_sign_negative() {
//         return b;
//     }
//     let m = a.max(b);
//     m + ((a - m).exp() + (b - m).exp()).ln()
// }

// // evaluator: discrete factors only (DiscreteTable)
// pub struct LogInfoEval;

// impl LogInfoEval {
//     fn logp_row(model: &FactorGraph, row: &[Option<VARVAL>]) -> f64 {
//         // build full joint over missing by enumeration
//         let n = model.variables.len();
//         let cards = cards(&model.variables);

//         // observed as indices
//         let mut obs = vec![None; n];
//         for i in 0..n {
//             obs[i] = match row[i] {
//                 Some(VARVAL::Discrete(s)) => Some(s),
//                 Some(VARVAL::Real(val)) => {
//                     panic!(
//                         "LogInfoEval only supports discrete variables, found Real value {}",
//                         val
//                     );
//                 }
//                 None => None,
//             }
//         }

//         println!("obs: {:?}", obs);

//         // unknown variable ids
//         let unknown = (0..n).filter(|&i| obs[i].is_none()).collect::<Vec<_>>();
//         if unknown.is_empty() {
//             let mut full = vec![0; n];
//             for i in 0..n {
//                 full[i] = obs[i].unwrap();
//             }

//             return Self::logp_assignment(model, &full);
//         }

//         // enumerate unknowns
//         let mut log_total = f64::NEG_INFINITY;
//         // multi-cartesian product over unknown states
//         let mut counters = vec![0; unknown.len()];
//         let dims = unknown.iter().map(|&i| cards[i]).collect::<Vec<_>>();

//         loop {
//             let mut full = vec![0; n];
//             for i in 0..n {
//                 full[i] = obs[i].unwrap_or(0);
//             }

//             for (k, &vid) in unknown.iter().enumerate() {
//                 full[vid] = counters[k];
//             }

//             println!("\n\nFull assignment: {:?}", full);

//             let lp = Self::logp_assignment(model, &full);
//             log_total = log_sum_exp(log_total, lp);

//             // increment mixed-radix counter
//             let mut carry = true;
//             for k in 0..counters.len() {
//                 if !carry {
//                     break;
//                 }

//                 counters[k] += 1;
//                 if counters[k] == dims[k] {
//                     counters[k] = 0;
//                     carry = true;
//                 } else {
//                     carry = false;
//                 }
//             }

//             if carry {
//                 break;
//             }
//         }

//         log_total
//     }

//     fn logp_assignment(model: &FactorGraph, assign: &[usize]) -> f64 {
//         let cards = cards(&model.variables);
//         let mut sum = 0f64;
//         for (incoming, f) in &model.factors {
//             match &f.potential {
//                 Potential::DiscreteTable(log_table) => {
//                     let idx = assignment_index(&incoming, assign, &cards);
//                     sum += log_table[idx] as f64;
//                 }
//                 _ => return f64::NEG_INFINITY, // not handled in discrete MVP
//             }
//         }
//         sum
//     }
// }

// impl Infer for LogInfoEval {
//     fn log_likelihood(&self, model: &FactorGraph, data: &ProbDataset) -> f32 {
//         let ll: f64 = data
//             .observations
//             .iter()
//             .map(|row| Self::logp_row(model, row))
//             .sum();
//         ll as f32
//     }
// }

// //        let mut pairs = Vec::new();

// //         let mut i = 0;
// //         for v in vertices.iter() {
// //             let allowed_inputs = inputs
// //                 .iter()
// //                 .enumerate()
// //                 .filter(|(idx, inp)| {
// //                     if let Some(val) = v.value().value() {
// //                         if inp
// //                             .value()
// //                             .domain()
// //                             .map(|d| d.is_categorical())
// //                             .unwrap_or(false)
// //                         {
// //                             let k = inp
// //                                 .value()
// //                                 .domain()
// //                                 .and_then(|d| match d {
// //                                     Domain::Categorical(k) => Some(*k),
// //                                     _ => None,
// //                                 })
// //                                 .unwrap();

// //                             if val.shape().map(|s| s.contains_dim(k - 1)).unwrap_or(false) {
// //                                 println!(
// //                                     "Vertex {} allows input {} (Cat dim {})",
// //                                     v.index(),
// //                                     inp.index(),
// //                                     k
// //                                 );
// //                                 return true;
// //                             } else {
// //                                 return false;
// //                             }
// //                         }
// //                     }

// //                     false
// //                 })
// //                 .map(|(idx, _)| inputs[idx].index())
// //                 .collect::<Vec<_>>();

// //             println!(
// //                 "Vertex {} allowed inputs: {:?}",
// //                 v.index(),
// //                 allowed_inputs.len()
// //             );

// //             pairs.push((i, allowed_inputs));

// //             // agg = agg.one_to_one(&allowed_inputs, v);
// //         }

// // //  let builder = NodeBuilder::new(&temp_store);
// // //     let mut agg = GraphAggregate::default();
// // //     let inputs = builder.input(self.cfg.variables.len());

// // //     let vertex = builder.vertices(self.cfg.num_factors); //self.factor_size(&scope));
// // //     let output = builder.output(1);

// // //     println!("Vertex node store: {:#?}", temp_store);

// // //     // for node in &vertex {
// // //     //     println!("Vertex node allele: {:#?}", node);
// // //     // }

// // //     // agg = agg.insert(&inputs);

// // //     // for ve in vertex.iter() {
// // //     //     let inputs = inputs
// // //     //         .iter()
// // //     //         .filter(|node| {
// // //     //             node.allele()
// // //     //                 .shape()
// // //     //                 .map(|s| s.contains(&ve.index()))
// // //     //                 .unwrap_or_default()
// // //     //         })
// // //     //         .cloned();

// // //     //     // agg = agg.many_to_one(&inputs.collect::<Vec<_>>(), ve);
// // //     // }

// // //     agg = agg.insert(&inputs);
// // //     // agg = agg.many_to_one(&inputs, &vertex);
// // //     // // agg = agg.fill(&inputs, &vertex);
// // //     // agg = agg.many_to_one(&vertex, &output);

// // //     let g = agg.build();

// // //     // let g = GraphChromosome::new(agg.build().into_iter().collect(), temp_store.clone());
// // //     // let g = Graph::new(g.new_instance(Some(temp_store.clone())).take_nodes());

// // //     // println!("Graph structure: {g:#?}");

// // //     // for node in g.iter() {
// // //     //     println!("Node {}: {:?}", node.index(), node.allele().is_valid());
// // //     // }

// // //     // let prob_dataset = ProbDataset::new(vec![
// // //     //     vec![Some(2), Some(0), Some(0)],
// // //     //     vec![Some(1), Some(0), Some(0)],
// // //     //     vec![Some(0), Some(1), Some(1)],
// // //     // ]);

// // //     // for row in &prob_dataset.observations {
// // //     //     let t = g.eval(&[row
// // //     //         .iter()
// // //     //         .map(|v| {
// // //     //             v.clone()
// // //     //                 .map(|val| match val {
// // //     //                     Value::Discrete(i) => i as f32,
// // //     //                     Value::Real(f) => 1.0,
// // //     //                 })
// // //     //                 .unwrap()
// // //     //         })
// // //     //         .collect::<Vec<_>>()]);

// // //     //     println!("Eval input {:?} -> output {:?}", row, t);
// // //     // }

// // //     // // println!("Initial graph eval output: {:?}", temp);

// // //     // let ll: f64 = prob_dataset
// // //     //     .observations
// // //     //     .iter()
// // //     //     .map(|row| {
// // //     //         row.iter()
// // //     //             .filter_map(|val| {
// // //     //                 val.clone().map(|v| match v {
// // //     //                     Value::Discrete(i) => Some(i),
// // //     //                     Value::Real(f) => Some(f as usize),
// // //     //                 })
// // //     //             })
// // //     //             .collect::<Vec<_>>()
// // //     //     })
// // //     //     .map(|row| logp_row_graph(&g, &row) as f64)
// // //     //     .sum();

// // //     // println!("Initial log-likelihood of random graph: {}", ll);
