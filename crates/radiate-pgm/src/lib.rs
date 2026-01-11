// Variables: define “what can vary” (Late, Rain, Time) and their state spaces (2/2/3).
//
// Factors: define “how they interact.” One ternary factor encodes the CPT
// (“if raining and 8am then Late is likely”), two unary factors are priors.
//
// Constraints: max_scope controls model complexity; num_factors controls how
// many relations you allow; cardinalities define the granularity of each variable.

// Gene: GraphNode<PgmNodeValue>
// Chromosome: GraphChromosome<PgmNodeValue>
// Codec: PgmCodec implementing Codec<PgmChromosome, FactorGraph>
// This slots directly into GeneticEngineBuilder with your existing pipeline.

mod factor;
mod value;
mod variable;

pub use factor::{FactorSpec, Potential};
pub use value::PgmValue;
pub use variable::{Domain, Value, Variable};

use radiate_core::{Chromosome, Codec, Gene, Genotype, random_provider};
use radiate_gp::{
    GraphNode, NodeStore, NodeValue,
    collections::{GraphChromosome, NodeType},
};
use radiate_utils::SortedBuffer;
use std::collections::BTreeSet;

pub struct ProbDataset {
    pub observations: Vec<Vec<Option<Value>>>, // length == num_vars per row
    pub num_vars: usize,
}

impl ProbDataset {
    pub fn new(observations: Vec<Vec<Option<impl Into<Value>>>>) -> Self {
        let obs = observations
            .into_iter()
            .map(|row| row.into_iter().map(|v| v.map(|val| val.into())).collect())
            .collect::<Vec<Vec<Option<Value>>>>();
        let num_vars = obs.first().map(|r| r.len()).unwrap_or(0);
        assert!(
            obs.iter().all(|r| r.len() == num_vars),
            "All rows must have same width"
        );
        Self {
            observations: obs,
            num_vars,
        }
    }

    pub fn rows(&self) -> usize {
        self.observations.len()
    }
}

/// A factor graph representing a probabilistic graphical model.
#[derive(Clone, Debug, PartialEq)]
pub struct FactorGraph {
    pub variables: Vec<Variable>,
    pub factors: Vec<(SortedBuffer<usize>, FactorSpec)>,
}

// Discrete-only: exact variable elimination / sum-product.
// Pure Gaussian (all Real with Gaussian/LinearGaussian): exact Gaussian elimination.
// Conditional Linear Gaussian (CLG) hybrid: exact if the BN meets CLG constraints (continuous nodes have linear-Gaussian CPDs; discrete nodes do not have continuous parents). Use per-discrete-parent-component Gaussians (MixtureByDiscrete).
// General hybrid: approximate (Gibbs/Metropolis, loopy BP with continuous messages, or EP).
// API stays the same; backend chooses by inspecting domains/potentials:

pub trait Infer {
    fn log_likelihood(&self, model: &FactorGraph, data: &ProbDataset) -> f32;
    // optional:
    // fn marginal(&self, var: usize, evidence: &[Option<Value>]) -> Distribution;
}

#[derive(Clone, Debug)]
pub struct PgmCodecConfig {
    // Explicit variables (ids must be 0..N-1 in order)
    pub variables: Vec<Variable>,

    // How many factors to initialize
    pub num_factors: usize,

    // Max number of variables allowed in any factor scope
    pub max_scope: usize,

    // Enable hybrid potentials
    pub allow_linear_gaussian: bool, // Real child with Real parents
    pub allow_mixture_by_discrete: bool, // Real child with Discrete parents
}

pub struct PgmCodec {
    cfg: PgmCodecConfig,
    node_store: NodeStore<PgmValue>,
}

impl PgmCodec {
    pub fn new(variables: Vec<impl Into<Variable>>, num_factors: usize, max_scope: usize) -> Self {
        let variables = variables.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let cfg = PgmCodecConfig {
            variables: variables.clone(),
            num_factors,
            max_scope,
            allow_linear_gaussian: true,
            allow_mixture_by_discrete: true,
        };
        Self {
            cfg,
            node_store: Self::create_node_store(variables, num_factors, max_scope),
        }
    }

    fn create_node_store(
        variables: Vec<impl Into<Variable>>,
        num_factors: usize,
        max_scope: usize,
    ) -> NodeStore<PgmValue> {
        let temp = vec![(
            NodeType::Input,
            variables
                .into_iter()
                .map(|v| PgmValue::Variable(v.into()))
                .collect::<Vec<_>>(),
        )];

        NodeStore::from(temp)
    }

    fn card_of(&self, var_id: usize) -> Option<usize> {
        match self.cfg.variables[var_id].domain {
            Domain::Discrete(k) => Some(k),
            Domain::Real => None,
        }
    }

    fn is_discrete_scope(&self, scope: &[usize]) -> bool {
        scope
            .iter()
            .all(|&v| matches!(self.cfg.variables[v].domain, Domain::Discrete(_)))
    }

    fn pick_linear_gaussian(&self, scope: &[usize]) -> Option<(usize, Vec<usize>)> {
        // Choose one Real child and Real parents (>=0)
        let real_vars: Vec<usize> = scope
            .iter()
            .copied()
            .filter(|&v| matches!(self.cfg.variables[v].domain, Domain::Real))
            .collect::<Vec<usize>>();

        if real_vars.is_empty() {
            return None;
        }

        // Child = first real; parents = remaining real in scope (could be empty)
        let child = real_vars[0];
        let parents = real_vars.iter().copied().filter(|&v| v != child).collect();

        Some((child, parents))
    }

    fn pick_mixture_by_discrete(&self, scope: &[usize]) -> Option<usize> {
        // Exactly one Real child and >=1 Discrete parents
        let real_vars: Vec<usize> = scope
            .iter()
            .copied()
            .filter(|&v| matches!(self.cfg.variables[v].domain, Domain::Real))
            .collect();

        if real_vars.len() != 1 {
            return None;
        }

        let has_discrete_parent = scope
            .iter()
            .any(|&v| matches!(self.cfg.variables[v].domain, Domain::Discrete(_)));

        if !has_discrete_parent {
            return None;
        }

        Some(real_vars[0]) // child
    }

    fn random_scope(&self) -> SortedBuffer<usize> {
        let num_vars = self.cfg.variables.len();
        let max_k = self.cfg.max_scope.min(num_vars).max(1);
        let k = random_provider::range(1..max_k + 1);

        let mut s = BTreeSet::new();
        while s.len() < k {
            s.insert(random_provider::range(0..num_vars));
        }

        s.into_iter().collect()
    }

    fn init_discrete_table(&self, scope: &[usize]) -> Potential {
        // Uniform log-table
        let size = scope.iter().fold(1, |acc, &v| {
            acc * self.card_of(v).expect("Expected Discrete domain")
        });

        let val = -((size as f64).ln()) as f32;
        Potential::DiscreteTable(vec![val; size])
    }

    fn init_gaussian(&self) -> Potential {
        let mean = random_provider::range(-1.0..1.0) as f32;
        let var = random_provider::range(0.1..1.0) as f32; // keep positive

        Potential::Gaussian(mean, var)
    }

    fn init_linear_gaussian(&self, parents: usize) -> Potential {
        let weights = (0..parents)
            .map(|_| random_provider::range(-1.0..1.0) as f64)
            .collect::<Vec<_>>();

        let bias = random_provider::range(-0.5..0.5) as f64;
        let var = random_provider::range(0.05..0.5) as f64;

        Potential::LinearGaussian { weights, bias, var }
    }

    fn init_mixture_by_discrete(&self, scope: &[usize], child: usize) -> Potential {
        // Components per discrete parent assignment
        let parents = scope
            .iter()
            .copied()
            .filter(|&v| v != child)
            .collect::<Vec<_>>();

        let comps = parents.iter().fold(1, |acc, &v| {
            acc * self.card_of(v).expect("Mixture requires discrete parents")
        });

        let mean = (0..comps)
            .map(|_| random_provider::range(-1.0..1.0) as f64)
            .collect::<Vec<_>>();
        let var = (0..comps)
            .map(|_| random_provider::range(0.05..0.5) as f64)
            .collect::<Vec<_>>();

        Potential::MixtureByDiscrete { mean, var }
    }

    fn make_factor_potential(&self, scope: &[usize]) -> Potential {
        // Priority: exact families when compatible; otherwise fall back to discrete/gaussian
        if scope.len() == 1 {
            // Unary: prior
            let v = scope[0];
            return match self.cfg.variables[v].domain {
                Domain::Discrete(_) => self.init_discrete_table(scope),
                Domain::Real => self.init_gaussian(),
            };
        }

        // Multi-var scopes
        if self.is_discrete_scope(scope) {
            return self.init_discrete_table(scope);
        }

        // Hybrid possibilities
        if self.cfg.allow_mixture_by_discrete {
            if let Some(child) = self.pick_mixture_by_discrete(scope) {
                return self.init_mixture_by_discrete(scope, child);
            }
        }
        if self.cfg.allow_linear_gaussian {
            if let Some((_child, parents_real)) = self.pick_linear_gaussian(scope) {
                return self.init_linear_gaussian(parents_real.len());
            }
        }

        // Fallbacks:
        // - If there is any Real and any Discrete but neither hybrid enabled, choose a unary prior
        //   for a random variable as a simple starting factor (keeps model valid).
        if scope
            .iter()
            .any(|&v| matches!(self.cfg.variables[v].domain, Domain::Real))
            && scope
                .iter()
                .any(|&v| matches!(self.cfg.variables[v].domain, Domain::Discrete(_)))
        {
            let pick = scope[random_provider::range(0..scope.len())];
            return match self.cfg.variables[pick].domain {
                Domain::Discrete(_) => self.init_discrete_table(&[pick]),
                Domain::Real => self.init_gaussian(),
            };
        }

        // Otherwise default to a Gaussian-like unary for a Real-only scope
        self.init_gaussian()
    }
}

impl Codec<GraphChromosome<PgmValue>, FactorGraph> for PgmCodec {
    fn encode(&self) -> Genotype<GraphChromosome<PgmValue>> {
        let num_variables = self.cfg.variables.len();
        let num_factors = self.cfg.num_factors;

        // 1) Sample factor scopes
        let mut scopes = Vec::with_capacity(num_factors);
        for _ in 0..num_factors {
            scopes.push(self.random_scope());
        }

        let mut factors = Vec::with_capacity(num_factors);
        for scope in scopes.iter() {
            let potential = self.make_factor_potential(&scope);
            factors.push(FactorSpec { potential });
        }

        // 3) Build nodes: variables [0..n), factors [n..n+m)
        let mut nodes = Vec::with_capacity(num_variables + num_factors);

        // 1) Variable nodes
        for var_id in 0..num_variables {
            // list factor indices that include this variable
            let factor_idxs = scopes
                .iter()
                .enumerate()
                .filter(|(_, sc)| sc.contains(&var_id))
                .map(|(j, _)| num_variables + j)
                .collect::<SortedBuffer<usize>>();

            let allele = PgmValue::Variable(self.cfg.variables[var_id].clone());
            nodes.push(
                GraphNode::from((var_id, NodeType::Input, allele))
                    .with_outgoing(factor_idxs.iter().cloned().into_iter()),
            );
        }

        // 2) Factor nodes
        for (j, scope) in scopes.iter().enumerate() {
            let idx = num_variables + j;

            let allele = PgmValue::Factor(FactorSpec {
                potential: factors[j].potential.clone(),
            });

            nodes.push(GraphNode::from((
                idx,
                NodeType::Vertex,
                allele,
                scope.clone(),
                scope.clone(),
            )));
        }

        Genotype::from(GraphChromosome::from(nodes))
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<PgmValue>>) -> FactorGraph {
        let mut variables = Vec::new();
        let mut factors = Vec::new();

        for (idx, gene) in genotype[0].iter().enumerate() {
            match gene.allele() {
                PgmValue::Variable(v) => variables.push((idx, v.clone())),
                PgmValue::Factor(f) => {
                    factors.push((gene.incoming_buffer().clone(), f.clone()));
                }
            }
        }

        variables.sort_by_key(|(idx, _)| *idx);
        let variables = variables.into_iter().map(|(_, v)| v).collect();
        FactorGraph { variables, factors }
    }
}

// helpers
fn cards(vars: &[Variable]) -> Vec<usize> {
    vars.iter()
        .map(|v| match v.domain {
            Domain::Discrete(k) => k,
            Domain::Real => 0,
        })
        .collect()
}

fn assignment_index(scope: &[usize], assign: &[usize], cards: &[usize]) -> usize {
    // row-major in scope order
    let mut idx = 0usize;
    let mut stride = 1usize;
    for (pos, &vid) in scope.iter().enumerate() {
        if pos > 0 {
            stride *= cards[scope[pos - 1]];
        }
        idx += assign[vid] * stride;
    }
    idx
}

fn log_sum_exp(a: f64, b: f64) -> f64 {
    if a.is_infinite() && a.is_sign_negative() {
        return b;
    }
    let m = a.max(b);
    m + ((a - m).exp() + (b - m).exp()).ln()
}

// evaluator: discrete factors only (DiscreteTable)
pub struct ExactDiscrete;

impl ExactDiscrete {
    fn logp_row(model: &FactorGraph, row: &[Option<Value>]) -> f64 {
        // build full joint over missing by enumeration
        let n = model.variables.len();
        let cards = cards(&model.variables);

        // observed as indices
        let mut obs: Vec<Option<usize>> = vec![None; n];
        for i in 0..n {
            obs[i] = match row[i] {
                Some(Value::Discrete(s)) => Some(s),
                Some(Value::Real(_)) => return f64::NEG_INFINITY, // not handled in discrete MVP
                None => None,
            }
        }

        // unknown variable ids
        let unknown: Vec<usize> = (0..n).filter(|&i| obs[i].is_none()).collect();
        if unknown.is_empty() {
            let mut full = vec![0usize; n];
            for i in 0..n {
                full[i] = obs[i].unwrap();
            }
            return Self::logp_assignment(model, &full);
        }

        // enumerate unknowns
        let mut log_total = f64::NEG_INFINITY;
        // multi-cartesian product over unknown states
        let mut counters = vec![0usize; unknown.len()];
        let dims: Vec<usize> = unknown.iter().map(|&i| cards[i]).collect();

        loop {
            let mut full = vec![0usize; n];
            for i in 0..n {
                full[i] = obs[i].unwrap_or(0);
            }
            for (k, &vid) in unknown.iter().enumerate() {
                full[vid] = counters[k];
            }

            let lp = Self::logp_assignment(model, &full);
            log_total = log_sum_exp(log_total, lp);

            // increment mixed-radix counter
            let mut carry = true;
            for k in 0..counters.len() {
                if !carry {
                    break;
                }
                counters[k] += 1;
                if counters[k] == dims[k] {
                    counters[k] = 0;
                    carry = true;
                } else {
                    carry = false;
                }
            }
            if carry {
                break;
            } // wrapped around
        }

        log_total
    }

    fn logp_assignment(model: &FactorGraph, assign: &[usize]) -> f64 {
        let cards = cards(&model.variables);
        let mut sum = 0f64;
        for (incoming, f) in &model.factors {
            match &f.potential {
                Potential::DiscreteTable(log_table) => {
                    let idx = assignment_index(&incoming, assign, &cards);
                    sum += log_table[idx] as f64;
                }
                _ => return f64::NEG_INFINITY, // not handled in discrete MVP
            }
        }
        sum
    }
}

impl Infer for ExactDiscrete {
    fn log_likelihood(&self, model: &FactorGraph, data: &ProbDataset) -> f32 {
        let ll: f64 = data
            .observations
            .iter()
            .map(|row| Self::logp_row(model, row))
            .sum();
        ll as f32
    }
}

// use super::domain::Domain;

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct Variable {
//     pub id: usize,
//     pub name: Option<String>,
//     pub domain: Domain,
// }

// impl Variable {
//     pub fn new(id: usize, domain: Domain) -> Self {
//         Self { id, name: None, domain }
//     }
// }

// #[derive(Clone, Debug, PartialEq)]
// pub struct FactorSpec {
//     // variable ids included in this factor, ordered
//     pub scope: Vec<usize>,
//     // dense log-potentials, length == product of scope cardinalities (row-major by scope order)
//     pub log_table: Vec<f64>,
// }

// impl FactorSpec {
//     pub fn uniform(scope: Vec<usize>, cardinalities: &[usize]) -> Self {
//         let size = scope.iter().map(|&v| cardinalities[v]).product::<usize>();
//         let value = -((size as f64).ln());
//         Self { scope, log_table: vec![value; size] }
//     }

//     pub fn param_count(&self, cardinalities: &[usize]) -> usize {
//         // For a fully-parameterized normalized table, independent params = table_size - 1
//         let size = self.scope.iter().map(|&v| cardinalities[v]).product::<usize>();
//         size.saturating_sub(1)
//     }
// }

// use super::{variable::Variable, factor::FactorSpec, domain::Domain};

// #[derive(Clone, Debug, PartialEq)]
// pub struct FactorGraph {
//     pub variables: Vec<Variable>,
//     pub factors: Vec<FactorSpec>,
// }

// impl FactorGraph {
//     pub fn cardinalities(&self) -> Vec<usize> {
//         self.variables.iter().map(|v| v.domain.cardinality()).collect()
//     }

//     pub fn num_vars(&self) -> usize {
//         self.variables.len()
//     }
// }

// // Each observation is a vector of Option<state_index> with length = num_vars.
// // None represents missing (to be marginalized).
// #[derive(Clone, Debug, PartialEq)]
// pub struct Dataset {
//     pub observations: Vec<Vec<Option<usize>>>,
//     pub num_vars: usize,
// }

// impl Dataset {
//     pub fn new(observations: Vec<Vec<Option<usize>>>) -> Self {
//         let num_vars = observations.first().map(|r| r.len()).unwrap_or(0);
//         assert!(observations.iter().all(|r| r.len() == num_vars), "All rows must have same width");
//         Self { observations, num_vars }
//     }

//     pub fn rows(&self) -> usize {
//         self.observations.len()
//     }
// }

// use crate::model::{variable::Variable, factor::FactorSpec};

// #[derive(Clone, Debug, PartialEq)]
// pub enum PgmValue {
//     Variable(Variable),
//     Factor(FactorSpec),
// }

// impl Default for PgmNodeValue {
//     fn default() -> Self {
//         PgmNodeValue::Variable(Variable { id: 0, name: None, domain: super::super::model::domain::Domain::Discrete { cardinality: 2 } })
//     }
// }

// impl PgmNodeValue {
//     pub fn is_variable(&self) -> bool {
//         matches!(self, PgmNodeValue::Variable(_))
//     }
//     pub fn is_factor(&self) -> bool {
//         matches!(self, PgmNodeValue::Factor(_))
//     }
// }

// use radiate_gp::collections::graphs::GraphChromosome;
// use super::node::PgmNodeValue;

// pub type PgmChromosome = GraphChromosome<PgmNodeValue>;

// use rand::{rngs::StdRng, SeedableRng, Rng};
// use radiate_core::{Codec, Genotype, Chromosome};
// use radiate_gp::collections::{GraphNode, NodeType};
// use crate::model::{factor_graph::FactorGraph, variable::Variable, domain::Domain, factor::FactorSpec};
// use super::{node::PgmNodeValue, chromosome::PgmChromosome};
// use std::collections::BTreeSet;

// #[derive(Clone, Debug)]
// pub struct PgmCodecConfig {
//     pub num_vars: usize,
//     pub cardinalities: Vec<usize>,  // length == num_vars
//     pub num_factors: usize,
//     pub max_scope: usize,           // >= 2
//     pub seed: u64,
// }

// impl PgmCodecConfig {
//     pub fn new(num_vars: usize, card: usize, num_factors: usize, max_scope: usize, seed: u64) -> Self {
//         Self {
//             num_vars,
//             cardinalities: vec![card; num_vars],
//             num_factors,
//             max_scope,
//             seed,
//         }
//     }
// }

// pub struct PgmCodec {
//     cfg: PgmCodecConfig,
// }

// impl PgmCodec {
//     pub fn new(cfg: PgmCodecConfig) -> Self {
//         assert!(cfg.num_vars > 0);
//         assert!(cfg.cardinalities.len() == cfg.num_vars);
//         assert!(cfg.max_scope >= 2);
//         assert!(cfg.num_factors > 0);
//         Self { cfg }
//     }

//     fn uniform_factor(scope: &[usize], cards: &[usize]) -> FactorSpec {
//         FactorSpec::uniform(scope.to_vec(), cards)
//     }
// }

// impl Codec<PgmChromosome, FactorGraph> for PgmCodec {
//     fn encode(&self) -> Genotype<PgmChromosome> {
//         let n = self.cfg.num_vars;
//         let m = self.cfg.num_factors.min(n.saturating_mul(2).max(1));
//         let cards = &self.cfg.cardinalities;

//         let mut rng = StdRng::seed_from_u64(self.cfg.seed);

//         // Plan factor scopes
//         let mut factor_scopes: Vec<Vec<usize>> = Vec::with_capacity(m);
//         for _ in 0..m {
//             let scope_len = rng.gen_range(2..=self.cfg.max_scope.min(n));
//             let mut scope: BTreeSet<usize> = BTreeSet::new();
//             while scope.len() < scope_len {
//                 scope.insert(rng.gen_range(0..n));
//             }
//             factor_scopes.push(scope.into_iter().collect());
//         }

//         // Variable neighbors (factors)
//         let mut var_to_factors: Vec<Vec<usize>> = vec![vec![]; n];
//         for (j, scope) in factor_scopes.iter().enumerate() {
//             let f_idx = n + j;
//             for &v in scope {
//                 var_to_factors[v].push(f_idx);
//             }
//         }

//         // Build variable nodes (Vertex with edges to factors)
//         let mut nodes: Vec<GraphNode<PgmNodeValue>> = Vec::with_capacity(n + m);
//         for v in 0..n {
//             let incoming = var_to_factors[v].clone();
//             let outgoing = var_to_factors[v].clone();
//             let value = PgmNodeValue::Variable(Variable::new(v, Domain::Discrete { cardinality: cards[v] }));
//             let node = GraphNode::from((v, NodeType::Vertex, value, incoming, outgoing));
//             nodes.push(node);
//         }

//         // Build factor nodes (Vertex with edges to variables in scope)
//         for (j, scope) in factor_scopes.iter().enumerate() {
//             let incoming = scope.clone();
//             let outgoing = scope.clone();
//             let table = Self::uniform_factor(scope, cards);
//             let value = PgmNodeValue::Factor(table);
//             let idx = n + j;
//             let node = GraphNode::from((idx, NodeType::Vertex, value, incoming, outgoing));
//             nodes.push(node);
//         }

//         // One chromosome (graph) genotype
//         let chromosome = PgmChromosome::from(nodes);
//         Genotype::new(vec![chromosome])
//     }

//     fn decode(&self, genotype: &Genotype<PgmChromosome>) -> FactorGraph {
//         let chr = &genotype[0];
//         let mut variables: Vec<Variable> = vec![];
//         let mut factors: Vec<FactorSpec> = vec![];

//         for gene in chr.iter() {
//             match gene.allele() {
//                 PgmNodeValue::Variable(v) => variables.push(v.clone()),
//                 PgmNodeValue::Factor(f) => factors.push(f.clone()),
//             }
//         }

//         // Sort variables by id to ensure stable order
//         variables.sort_by_key(|v| v.id);
//         FactorGraph { variables, factors }
//     }
// }

// use crate::model::{factor_graph::FactorGraph, dataset::Dataset};

// pub trait Infer {
//     fn log_likelihood(&self, model: &FactorGraph, data: &Dataset) -> f64;
// }

// use itertools::Itertools;
// use crate::model::{factor_graph::FactorGraph, dataset::Dataset, factor::FactorSpec};
// use super::traits::Infer;

// pub struct ExactEnumerator;

// impl ExactEnumerator {
//     fn assignment_index(scope: &[usize], assign: &[usize], cards: &[usize]) -> usize {
//         // row-major by scope order
//         let mut stride = 1usize;
//         let mut idx = 0usize;
//         for (k, &var) in scope.iter().enumerate() {
//             if k > 0 {
//                 stride *= cards[scope[k - 1]];
//             }
//             idx += assign[var] * stride;
//         }
//         idx
//     }

//     fn factor_logp(f: &FactorSpec, assign: &[usize], cards: &[usize]) -> f64 {
//         let idx = Self::assignment_index(&f.scope, assign, cards);
//         f.log_table[idx]
//     }

//     fn logp_full_assignment(model: &FactorGraph, assign: &[usize]) -> f64 {
//         let cards = model.cardinalities();
//         model.factors.iter().map(|f| Self::factor_logp(f, assign, &cards)).sum()
//     }

//     fn logp_evidence(model: &FactorGraph, ev: &[Option<usize>]) -> f64 {
//         let n = model.num_vars();
//         let cards = model.cardinalities();

//         let unknown: Vec<usize> = (0..n).filter(|&i| ev[i].is_none()).collect();
//         if unknown.is_empty() {
//             // Direct evaluation
//             let mut full = vec![0usize; n];
//             for i in 0..n {
//                 full[i] = ev[i].unwrap();
//             }
//             return Self::logp_full_assignment(model, &full);
//         }

//         let mut log_num = f64::NEG_INFINITY;
//         for states in unknown.iter().map(|&i| 0..cards[i]).multi_cartesian_product() {
//             let mut full = vec![0usize; n];
//             for i in 0..n {
//                 full[i] = ev[i].unwrap_or(0);
//             }
//             for (k, &i) in unknown.iter().enumerate() {
//                 full[i] = states[k];
//             }
//             let lp = Self::logp_full_assignment(model, &full);
//             log_num = log_sum_exp(log_num, lp);
//         }

//         log_num
//     }
// }

// fn log_sum_exp(a: f64, b: f64) -> f64 {
//     if a.is_infinite() && a.is_sign_negative() { return b; }
//     let m = a.max(b);
//     m + (a - m).exp().ln_1p((b - m).exp())
// }

// impl Infer for ExactEnumerator {
//     fn log_likelihood(&self, model: &FactorGraph, data: &Dataset) -> f64 {
//         data.observations.iter().map(|row| Self::logp_evidence(model, row)).sum()
//     }
// }

// use crate::model::{dataset::Dataset, factor_graph::FactorGraph};
// use crate::inference::{traits::Infer, exact::ExactEnumerator};
// use radiate_core::Score;

// pub fn nll(dataset: Dataset) -> impl Fn(FactorGraph) -> Score {
//     move |model: FactorGraph| {
//         let inf = ExactEnumerator;
//         let ll = inf.log_likelihood(&model, &dataset);
//         let nll = -ll as f32;
//         Score::from(nll)
//     }
// }

// // Simple BIC: -2*LL + k*ln(n)
// pub fn bic(dataset: Dataset) -> impl Fn(FactorGraph) -> Score {
//     move |model: FactorGraph| {
//         let inf = ExactEnumerator;
//         let ll = inf.log_likelihood(&model, &dataset);
//         let k: usize = {
//             let cards = model.cardinalities();
//             model.factors.iter().map(|f| f.param_count(&cards)).sum()
//         };
//         let n = dataset.rows().max(1) as f64;
//         let bic = -2.0 * ll + (k as f64) * n.ln();
//         Score::from(bic as f32)
//     }
// }

// use radiate_core::{Objective, Optimize, EngineProblem};
// use radiate_pgm::{PgmCodec, PgmCodecConfig, fitness, model::dataset::Dataset};
// use std::sync::Arc;

// // Build codec
// let cfg = PgmCodecConfig::new(num_vars, card, num_factors, max_scope, 42);
// let codec = PgmCodec::new(cfg);

// // Prepare dataset
// let ds = Dataset::new(vec![
//     vec![Some(0), None, Some(1)],
//     vec![Some(1), Some(0), None],
// ]);

// // Fitness (NLL)
// let fitness_fn = Arc::new(fitness::nll(ds));

// // Problem for builder when not using batch raw fitness
// let problem = EngineProblem {
//     objective: Objective::Single(Optimize::Minimize),
//     codec: Arc::new(codec),
//     fitness_fn: Some(fitness_fn),
//     raw_fitness_fn: None,
// };

// use radiate_pgm::{
//     // model
//     Domain, Variable, FactorSpec, FactorGraph, Dataset,
//     // inference
//     Infer, ExactEnumerator,
// };

// // Helper: stable index into a row-major log-table for scope [X1, X2, X3] with cards [2, 2, 3]
// fn idx_x1_x2_x3(x1: usize, x2: usize, x3: usize) -> usize {
//     // order: [X1, X2, X3]; cards: [2, 2, 3]
//     x1 + (2 * x2) + (2 * 2 * x3)
// }

// fn main() {
//     // 1) Variables and domains
//     // X1 = Late? {OnTime(0), Late(1)}; X2 = Raining? {No(0), Yes(1)}; X3 = Time {7am(0), 8am(1), 9am(2)}
//     let x1 = Variable { id: 0, name: Some("Late".into()), domain: Domain::Discrete { cardinality: 2 } };
//     let x2 = Variable { id: 1, name: Some("Rain".into()), domain: Domain::Discrete { cardinality: 2 } };
//     let x3 = Variable { id: 2, name: Some("Time".into()), domain: Domain::Discrete { cardinality: 3 } };
//     let variables = vec![x1.clone(), x2.clone(), x3.clone()];

//     // 2) Factors (log-space)
//     // f1(X1, X2, X3) encodes CPT P(X1 | X2, X3), row-normalized over X1 for each (X2, X3)
//     let mut f1_log = vec![f64::NEG_INFINITY; 2 * 2 * 3];

//     // baseline CPT rows (X1 | X2, X3): default [P(OnTime), P(Late)] = [0.8, 0.2]
//     for x2_state in 0..2 {
//         for x3_state in 0..3 {
//             let p_on = 0.8_f64;
//             let p_late = 0.2_f64;
//             f1_log[idx_x1_x2_x3(0, x2_state, x3_state)] = p_on.ln();
//             f1_log[idx_x1_x2_x3(1, x2_state, x3_state)] = p_late.ln();
//         }
//     }

//     // rule: if raining (X2=Yes=1) and 8am (X3=1) then P(Late=1) is high → [0.1, 0.9]
//     f1_log[idx_x1_x2_x3(0, 1, 1)] = 0.1_f64.ln();
//     f1_log[idx_x1_x2_x3(1, 1, 1)] = 0.9_f64.ln();

//     let f1 = FactorSpec { scope: vec![x1.id, x2.id, x3.id], log_table: f1_log };

//     // f2(X2) = P(X2) prior: [No, Yes] = [0.7, 0.3]
//     let f2 = FactorSpec {
//         scope: vec![x2.id],
//         log_table: vec![0.7_f64.ln(), 0.3_f64.ln()],
//     };

//     // f3(X3) = P(X3) prior: [7am, 8am, 9am] = [0.3, 0.4, 0.3]
//     let f3 = FactorSpec {
//         scope: vec![x3.id],
//         log_table: vec![0.3_f64.ln(), 0.4_f64.ln(), 0.3_f64.ln()],
//     };

//     // 3) Assemble factor graph
//     let fg = FactorGraph {
//         variables,
//         factors: vec![f1, f2, f3],
//     };

//     // 4) Query P(X1 | X2=Yes, X3=8am)
//     // Evidence row uses Option<usize>: None = unobserved
//     let evidence = vec![None, Some(1), Some(1)]; // [X1=?, X2=Yes(1), X3=8am(1)]
//     let ds = Dataset::new(vec![evidence]);

//     // Exact inference (enumeration for MVP)
//     let inf = ExactEnumerator;

//     // Compute unnormalized log for X1=0 and X1=1, then normalize manually
//     fn logsumexp(a: f64, b: f64) -> f64 {
//         if a.is_infinite() && a.is_sign_negative() { return b; }
//         let m = a.max(b);
//         m + (a - m).exp().ln_1p((b - m).exp())
//     }

//     // Build two rows: one with X1=0, one with X1=1
//     let ll_x1_0 = {
//         let mut obs = vec![Some(0), Some(1), Some(1)];
//         let tmp = Dataset::new(vec![obs]);
//         inf.log_likelihood(&fg, &tmp)
//     };
//     let ll_x1_1 = {
//         let mut obs = vec![Some(1), Some(1), Some(1)];
//         let tmp = Dataset::new(vec![obs]);
//         inf.log_likelihood(&fg, &tmp)
//     };

//     let log_den = logsumexp(ll_x1_0, ll_x1_1);
//     let p_x1_0 = (ll_x1_0 - log_den).exp();
//     let p_x1_1 = (ll_x1_1 - log_den).exp();

//     println!("P(X1=OnTime | X2=Yes, X3=8am) ≈ {:.3}", p_x1_0);
//     println!("P(X1=Late   | X2=Yes, X3=8am) ≈ {:.3}", p_x1_1);

//     // 5) Data likelihood example
//     // Suppose we observed that at 8am & raining, I was Late:
//     let ds_obs = Dataset::new(vec![vec![Some(1), Some(1), Some(1)]]);
//     let ll = inf.log_likelihood(&fg, &ds_obs);
//     println!("log-likelihood of the observation: {:.3}", ll);
// }
