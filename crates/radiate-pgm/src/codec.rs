use crate::{
    Arity, Graph, GraphAggregate, GraphChromosome, GraphNode, NodeBuilder, NodeStore, NodeType,
    NodeValue, Op,
};
use radiate_core::{Codec, Genotype, random_provider};

#[derive(Clone, Debug, PartialEq)]
pub enum FactorType {
    GaussLin2,
    Gauss1,
    Logp,
}

pub struct PgmCodec {
    num_factors: usize,
    max_scope: usize,
    store: NodeStore<Op<f32>>,
    factors: Vec<FactorType>,
}

impl PgmCodec {
    pub fn new(
        num_factors: usize,
        max_scope: usize,
        factors: Vec<FactorType>,
        store: impl Into<NodeStore<Op<f32>>>,
    ) -> Self {
        let store = store.into();
        let init_potential = Self::init_factors(&store, num_factors, max_scope, &factors);

        store.insert(NodeType::Vertex, init_potential);

        PgmCodec {
            num_factors,
            max_scope,
            store,
            factors,
        }
    }

    fn init_factors(
        store: &NodeStore<Op<f32>>,
        num_factors: usize,
        max_scope: usize,
        factors: &[FactorType],
    ) -> Vec<NodeValue<Op<f32>>> {
        let mut result = Vec::with_capacity(num_factors);

        let other =
            random_scopes_covering(store.count_type(NodeType::Input), num_factors, max_scope)
                .iter()
                .map(|scope| (scope.clone(), store.cards_for_scope(scope)))
                .collect::<Vec<(Vec<usize>, Vec<usize>)>>();

        for pair in other.iter() {
            let rand_factor = random_provider::choose(&factors);
            let factor_op = match rand_factor {
                FactorType::GaussLin2 => Op::<f32>::gauss_lin2(),
                FactorType::Gauss1 => Op::<f32>::gauss1(),
                FactorType::Logp => Op::<f32>::logprob_table(pair.1.as_slice().to_vec()),
            };

            let arity = factor_op.arity();
            result.push(NodeValue::Bounded(factor_op, arity));
        }

        result
    }

    fn build_factor_for_scope(&self, cards: &[usize]) -> Op<f32> {
        let scope_len = cards.len();

        let compatible = self
            .factors
            .iter()
            .filter(|ft| match ft {
                FactorType::Gauss1 => scope_len == 1,
                FactorType::GaussLin2 => scope_len == 2,
                FactorType::Logp => scope_len >= 1,
            })
            .collect::<Vec<_>>();

        let picked = if compatible.is_empty() {
            &FactorType::Logp
        } else {
            random_provider::choose(&compatible)
        };

        match picked {
            FactorType::GaussLin2 => Op::<f32>::gauss_lin2(),
            FactorType::Gauss1 => Op::<f32>::gauss1(),
            FactorType::Logp => Op::<f32>::logprob_table(cards.to_vec()),
        }
    }
}

impl Codec<GraphChromosome<Op<f32>>, Graph<Op<f32>>> for PgmCodec {
    fn encode(&self) -> Genotype<GraphChromosome<Op<f32>>> {
        let builder = NodeBuilder::new(self.store.clone());
        let mut agg = GraphAggregate::default();

        let scope_card_pairs = random_scopes_covering(
            self.store.count_type(NodeType::Input),
            self.num_factors,
            self.max_scope,
        )
        .iter()
        .map(|scope| (scope.clone(), self.store.cards_for_scope(scope)))
        .collect::<Vec<(Vec<usize>, Vec<usize>)>>();

        let inps = scope_card_pairs
            .iter()
            .enumerate()
            .map(|(idx, pair)| {
                (
                    pair.0.clone(),
                    GraphNode::from((
                        idx,
                        NodeType::Vertex,
                        self.build_factor_for_scope(pair.1.as_slice()),
                        Arity::Exact(pair.1.len()),
                    )),
                )
            })
            .collect::<Vec<_>>();

        let inputs = builder.input(inps.len());
        let output = builder.output(1);

        agg = agg.insert(&inputs);
        agg = agg.insert(&output);
        for (idxs, vertex) in inps.iter() {
            for &idx in idxs.iter() {
                agg = agg.one_to_one(&inputs[idx], vertex);
            }

            agg = agg.one_to_one(vertex, &output);
        }

        let mut graph = agg.build();

        Genotype::from(GraphChromosome::from((
            graph.take_nodes(),
            self.store.clone(),
        )))
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<Op<f32>>>) -> Graph<Op<f32>> {
        Graph::new(genotype[0].clone().into_iter().collect::<Vec<_>>())
    }
}

/// Random scopes, but guarantees every var 0..num_vars appears at least once.
/// Also avoids empty scopes and clamps scope sizes to [1..=max_scope].
pub fn random_scopes_covering(
    num_vars: usize,
    num_factors: usize,
    max_scope: usize,
) -> Vec<Vec<usize>> {
    assert!(num_vars > 0);
    assert!(num_factors > 0);

    let max_scope = max_scope.clamp(1, num_vars);

    // 1) initial random scopes
    let var_indices: Vec<usize> = (0..num_vars).collect();
    let mut scopes: Vec<Vec<usize>> = (0..num_factors)
        .map(|_| {
            let k = random_provider::range(1..max_scope + 1);
            let mut s = random_provider::sample_without_replacement(&var_indices, k);
            s.sort_unstable();
            s
        })
        .collect();

    // 2) repair coverage: ensure each var appears at least once
    let mut seen = vec![0usize; num_vars];
    for s in &scopes {
        for &v in s {
            seen[v] += 1;
        }
    }

    let mut seen_count = seen.iter().filter(|&&c| c > 0).count();
    if seen_count == num_vars {
        return scopes;
    }

    let mut iteration = 0;
    while seen_count < num_vars {
        iteration += 1;
        if iteration > 10 {
            panic!("Too many iterations in random_scopes_covering repair loop");
        }

        for v in 0..num_vars {
            if seen[v] > 0 {
                continue;
            }

            // pick a scope to inject into
            let j = random_provider::range(0..scopes.len());

            // if scope already full, replace a random element; else insert
            if scopes[j].len() >= max_scope {
                let r = random_provider::range(0..scopes[j].len());
                let old = scopes[j][r];
                scopes[j][r] = v;

                // update seen counts
                seen[old] = seen[old].saturating_sub(1);
                seen[v] += 1;
            } else {
                scopes[j].push(v);
                scopes[j].sort_unstable();
                scopes[j].dedup();
                seen[v] += 1;
            }
        }

        seen_count = seen.iter().filter(|&&c| c > 0).count();
    }

    scopes
}
