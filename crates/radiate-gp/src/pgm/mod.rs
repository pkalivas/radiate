use crate::{
    Arity, EvalMut, Factory, Graph, GraphAggregate, GraphChromosome, GraphEvaluator, GraphNode,
    NodeBuilder, NodeStore, NodeType, NodeValue, Op, node_store,
};
use radiate_core::{Codec, Genotype, random_provider};
use std::vec;

mod factor;
pub use factor::*;

impl<T> Factory<&[usize], Vec<(usize, usize)>> for NodeStore<PgmOp<T>> {
    fn new_instance(&self, scope: &[usize]) -> Vec<(usize, usize)> {
        self.map_by_type(NodeType::Input, |values| {
            values
                .into_iter()
                .filter_map(|value| match value.value() {
                    PgmOp::Variable(_, idx, card) if scope.contains(idx) => {
                        Some((*idx, card.unwrap_or(1)))
                    }
                    _ => None,
                })
                .collect::<Vec<(usize, usize)>>()
        })
        .unwrap_or_default()
    }
}

pub struct PgmCodec<T> {
    num_factors: usize,
    max_scope: usize,
    store: NodeStore<Op<T>>,
    pgm_store: NodeStore<PgmOp<T>>,
}

impl<T: Default + Clone> PgmCodec<T> {
    pub fn new(
        num_factors: usize,
        max_scope: usize,
        store: impl Into<NodeStore<PgmOp<T>>>,
    ) -> Self {
        let pgm_store = store.into();

        PgmCodec {
            num_factors,
            max_scope,
            store: node_store!(
                Input => pgm_store
                    .map_by_type(NodeType::Input, |values| {
                        values
                            .iter()
                            .map(|val| {
                                Op::<T>::from(val.value())
                            })
                            .collect::<Vec<Op<T>>>()
                    })
                    .unwrap_or_default(),
                Output => pgm_store
                    .map_by_type(NodeType::Output, |values| {
                        values
                            .iter()
                            .map(|val| Op::<T>::from(val.value()))
                            .collect::<Vec<Op<T>>>()
                    })
                    .unwrap_or_default()
            ),
            pgm_store,
        }
    }
}

impl Codec<GraphChromosome<Op<f32>>, Graph<Op<f32>>> for PgmCodec<f32> {
    fn encode(&self) -> Genotype<GraphChromosome<Op<f32>>> {
        let builder = NodeBuilder::new(self.store.clone());
        let mut agg = GraphAggregate::default();

        let other = self
            .store
            .random_scopes(self.num_factors, self.max_scope)
            .iter()
            .map(|scope| (scope.clone(), self.store.cards_for_scope(scope)))
            .collect::<Vec<(Vec<usize>, Vec<usize>)>>();

        let inps = other
            .iter()
            .enumerate()
            .filter_map(|(idx, pair)| {
                self.pgm_store.map_by_type(NodeType::Vertex, |values| {
                    let rand_val = random_provider::choose(values);

                    match rand_val {
                        NodeValue::Unbound(value) => Some((
                            pair.0.clone(),
                            GraphNode::from((
                                idx,
                                NodeType::Vertex,
                                match value {
                                    PgmOp::Factor(factor_type) => match factor_type {
                                        FactorType::GaussLin2 => Op::<f32>::gauss_lin2(),
                                        FactorType::Gauss1 => Op::<f32>::gauss1(),
                                        FactorType::Logp => {
                                            Op::<f32>::logprob_table(pair.1.as_slice().to_vec())
                                        }
                                    },
                                    _ => Op::default(),
                                },
                                Arity::Exact(pair.1.len()),
                            )),
                        )),
                        _ => None,
                    }
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        let inputs = builder.input(inps.len());

        let output = builder.output(1);

        agg = agg.insert(&inputs);
        for (idxs, vertex) in inps.iter() {
            for &idx in idxs.iter() {
                agg = agg.one_to_one(&inputs[idx], vertex);
            }

            agg = agg.one_to_one(vertex, &output);
        }

        let graph = agg.build();

        Genotype::from(vec![GraphChromosome::from((
            graph.into_iter().collect::<Vec<_>>(),
            self.store.clone(),
        ))])
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<Op<f32>>>) -> Graph<Op<f32>> {
        Graph::new(genotype[0].clone().into_iter().collect::<Vec<_>>())
    }
}

#[derive(Clone)]
pub struct PgmDataSet {
    rows: Vec<Vec<Option<usize>>>, // length = num_vars per row
}

impl PgmDataSet {
    pub fn new(rows: Vec<Vec<Option<usize>>>) -> Self {
        Self { rows }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Vec<Option<usize>>> {
        self.rows.iter()
    }
    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Clone)]
pub struct PgmLogLik {
    data: PgmDataSet,
    num_vars: usize,
}

impl PgmLogLik {
    pub fn new(data: PgmDataSet, num_vars: usize) -> Self {
        Self { data, num_vars }
    }

    #[inline]
    fn row_to_inputs<'a>(&self, row: &[Option<usize>], buf: &'a mut Vec<f32>) -> Option<&'a [f32]> {
        buf.clear();
        buf.reserve(self.num_vars);

        // require fully observed for v0
        if row.len() != self.num_vars {
            return None;
        }

        for v in row {
            let Some(x) = v else {
                return None;
            };

            buf.push(*x as f32);
        }

        Some(buf)
    }
}

thread_local! {
    static PGM_INP_BUF: std::cell::RefCell<Vec<f32>> = std::cell::RefCell::new(Vec::new());
}

impl radiate_core::fitness::FitnessFunction<Graph<Op<f32>>, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, graph: Graph<Op<f32>>) -> f32 {
        let mut ev = GraphEvaluator::new(&graph);

        let mut sum_ll = 0.0f32;
        let mut n = 0usize;

        for row in self.data.iter() {
            // let ll_opt = PGM_INP_BUF.with(|cell| {
            //     let mut buf = cell.borrow_mut();
            //     let inp = self.row_to_inputs(row, &mut buf)?;
            //     Some(ev.eval_mut(inp)[0])
            // });

            let ll_opt = PGM_INP_BUF.with(|cell| {
                let mut buf = cell.borrow_mut();
                let inp = self.row_to_inputs(row, &mut buf)?;
                Some(ev.eval_mut(inp)[0])
            });

            if let Some(ll) = ll_opt {
                sum_ll += ll;
                n += 1;
            }
        }

        if n == 0 {
            return 1e9;
        }

        // minimize negative average log-likelihood
        -(sum_ll / n as f32)
    }
}

// 1) Variable nodes
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

// let inputs = (0..num_vars)
//     .map(|i| Op::category(format!("{}", i), i, random_provider::range(2..5)))
//     .collect::<Vec<Op<f32>>>();

// let mut factors = vec![
//     Op::<f32>::gauss1(),
//     Op::<f32>::gauss_lin2(),
//     // optional:
//     // Op::<f32>::add(), Op::<f32>::mul(), ...
// ];

// let random_shapes = (1..=3)
//     .map(|i| {
//         let mut shape = vec![];

//         let sample = random_provider::sample_indices(0..num_vars, i);
//         for idx in sample {
//             let var_op = &inputs[idx];
//             if let Op::Var(_, _, Domain::Categorical(k)) = var_op {
//                 shape.push(*k);
//             }
//         }

//         Shape::new(shape)
//     })
//     .collect::<Vec<Shape>>();

// for sh in random_shapes {
//     factors.push(Op::<f32>::nd_array(sh.clone()));
//     factors.push(Op::<f32>::logprob_table(sh.clone()));
// }

// return node_store!(
//     Input => inputs,
//     Vertex => factors,
//     Output => vec![Op::<f32>::sum()]
// );

// fn create_graph(&self, scope: &[usize]) -> Graph<Op<T>>
// where
//     T: Default + Clone + Debug,
// {
//     let inputs = self.pgm_store.map_by_type(NodeType::Input, |values| {
//         values
//             .iter()
//             .enumerate()
//             .filter(|(idx, _)| scope.contains(idx))
//             .map(|val| match val.1 {
//                 NodeValue::Bounded(value, _) => GraphNode::from((
//                     val.0,
//                     NodeType::Input,
//                     match value {
//                         PgmValue::Variable(name, idx, domain) => {
//                             Op::Var(name, *idx, domain.clone())
//                         }
//                         _ => Op::default(),
//                     },
//                 )),
//                 NodeValue::Unbound(value) => GraphNode::from((
//                     val.0,
//                     NodeType::Input,
//                     match value {
//                         PgmValue::Variable(name, idx, domain) => {
//                             Op::Var(name, *idx, domain.clone())
//                         }
//                         _ => Op::default(),
//                     },
//                 )),
//             })
//             .collect::<Graph<Op<T>>>()
//     });

//     println!("SCOPE: {:?}", scope);

//     println!("INPUTS FROM STORE: {:?}", inputs);
//     // let vertex = self.pgm_store.map_by_type(NodeType::Vertex, |values| {
//     //     values
//     //         .iter()
//     //         .enumerate()
//     //         .filter_map(|(idx, val)| match val {
//     //             NodeValue::Unbound(value) => Some(GraphNode::from((
//     //                 idx,
//     //                 NodeType::Vertex,
//     //                 match value {
//     //                     PgmValue::Factor(factor_type) => match factor_type {
//     //                         FactorType::GaussLin2 => Op::<f32>::gauss_lin2(),
//     //                         FactorType::Gauss1 => Op::<f32>::gauss1(),
//     //                         FactorType::Logp => Op::<f32>::logprob_table(),
//     //                     },
//     //                     _ => Op::default(),
//     //                 },
//     //             ))),
//     //             _ => None,
//     //         })
//     //         .collect::<Graph<Op<f32>>>()
//     // });

//     if let Some(inputs) = inputs {
//         inputs
//     } else {
//         Graph::new(vec![])
//     }

//     // println!("INPUTS: {:?}", inputs);

//     // panic!("TODO: finish PGM codec query_node_store");
// }
