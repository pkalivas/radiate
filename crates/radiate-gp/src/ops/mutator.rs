use crate::Tree;
use crate::node::Node;
use crate::ops::operation::Op;
use crate::{
    Factory, GraphChromosome, NodeStore, NodeType, TreeChromosome, TreeIterator, TreeNode,
};
use radiate_core::genome::Gene;
use radiate_core::{AlterResult, Mutate};
use radiate_core::{Chromosome, random_provider};
use std::sync::{Arc, Mutex};

pub struct OperationMutator {
    rate: f32,
    replace_rate: f32,
}

impl OperationMutator {
    pub fn new(rate: f32, replace_rate: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("rate must be between 0.0 and 1.0");
        }

        if !(0.0..=1.0).contains(&replace_rate) {
            panic!("replace_rate must be between 0.0 and 1.0");
        }

        OperationMutator { rate, replace_rate }
    }
}

/// This implementation is for the `GraphChromosome<Op<T>>` type.
/// It mutates the chromosome by changing the value of the `MutableConst` Op nodes (weights).
/// If the node is not a `MutableConst` node, it tries to replace it with a new node from the store,
/// but only if the arity of the new node is the same as the current node.
impl<T> Mutate<GraphChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<Op<T>>, rate: f32) -> AlterResult {
        let mutation_indexes = (0..chromosome.len())
            .filter(|_| random_provider::random::<f32>() < rate)
            .collect::<Vec<usize>>();

        if mutation_indexes.is_empty() {
            return 0.into();
        }

        for i in mutation_indexes.iter() {
            let node = chromosome.get(*i);

            if matches!(node.node_type(), NodeType::Input | NodeType::Output) {
                continue;
            }

            if let Some(store) = chromosome.store().map(|store| store.clone()) {
                if let Some(new_op) = mutate_node(node, &store, self.replace_rate) {
                    (*chromosome.as_mut()[*i].value_mut()) = new_op;
                }
            }

            // match node.allele() {
            //     Op::MutableConst { .. } => {
            //         if let Some(new_op) = mutate_mutable_const(node, self.replace_rate) {
            //             (*chromosome.as_mut()[*i].value_mut()) = new_op;
            //         }
            //     }
            //     #[cfg(feature = "pgm")]
            //     Op::PGM(..) => {
            //         if let Some(store) = chromosome.store() {
            //             if let Some(new_op) = mutate_pgm_node(node, store, self.replace_rate) {
            //                 (*chromosome.as_mut()[*i].value_mut()) = new_op;
            //             }
            //         }
            //     }
            //     _ => {
            //         if let Some(store) = chromosome.store().map(|store| store.clone()) {
            //             let new_op = mutate_fallback_from_store(node, &store);

            //             if let Some(new_op) = new_op {
            //                 (*chromosome.as_mut()[*i].value_mut()) = new_op;
            //             }
            //         }
            //     }
            // }
        }

        mutation_indexes.len().into()
    }
}

impl<T> Mutate<TreeChromosome<Op<T>>> for OperationMutator
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut TreeChromosome<Op<T>>, rate: f32) -> AlterResult {
        let store = chromosome.get_store();
        if let Some(store) = store {
            let count = Arc::new(Mutex::new(0));
            let cloned_count = Arc::clone(&count);
            let root = chromosome.root_mut();

            root.apply(move |node| {
                if random_provider::random::<f32>() < rate {
                    (*cloned_count.lock().unwrap()) += match node.allele() {
                        Op::MutableConst { .. } => {
                            if let Some(new_op) = mutate_mutable_const(node, self.replace_rate) {
                                (*node.value_mut()) = new_op;
                                1
                            } else {
                                0
                            }
                        }
                        #[cfg(feature = "pgm")]
                        Op::PGM(..) => {
                            if let Some(new_op) = mutate_pgm_node(node, &store, self.replace_rate) {
                                (*node.value_mut()) = new_op;
                                1
                            } else {
                                0
                            }
                        }
                        _ => {
                            if let Some(new_op) = mutate_fallback_from_store(node, &store) {
                                (*node.value_mut()) = new_op;
                                1
                            } else {
                                0
                            }
                        }
                    };
                    // mutate_tree_node(node, &store, self.replace_rate);
                }
            });

            return (*count.lock().unwrap()).into();
        } else {
            0.into()
        }
    }
}

pub fn mutate_node<T>(
    node: &impl Node<Value = Op<T>>,
    store: &NodeStore<Op<T>>,
    replace_rate: f32,
) -> Option<Op<T>>
where
    T: Clone + PartialEq + Default,
{
    match node.value() {
        Op::MutableConst { .. } => mutate_mutable_const(node, replace_rate),
        #[cfg(feature = "pgm")]
        Op::PGM(..) => mutate_pgm_node(node, store, replace_rate),
        _ => mutate_fallback_from_store(node, store),
    }
}

pub fn mutate_mutable_const<T>(node: &impl Node<Value = Op<T>>, replace_rate: f32) -> Option<Op<T>>
where
    T: Clone + PartialEq + Default,
{
    match node.value() {
        Op::MutableConst {
            name,
            arity,
            value,
            supplier: get_value,
            modifier,
            operation,
        } => {
            let new_value = get_value();
            let new_value = if random_provider::random::<f32>() < replace_rate {
                new_value
            } else {
                modifier(value)
            };
            Some(Op::MutableConst {
                name,
                arity: *arity,
                value: new_value,
                modifier: *modifier,
                supplier: *get_value,
                operation: *operation,
            })
        }
        _ => None,
    }
}

pub fn mutate_fallback_from_store<T>(
    node: &impl Node<Value = Op<T>>,
    store: &NodeStore<Op<T>>,
) -> Option<Op<T>>
where
    T: Clone + PartialEq + Default,
{
    let new_op: Op<T> = store.new_instance(node.node_type());
    (new_op.arity() == node.value().arity()).then_some(new_op)
}

#[cfg(feature = "pgm")]
pub fn mutate_pgm_node<T>(
    node: &impl Node<Value = Op<T>>,
    store: &NodeStore<Op<T>>,
    replace_rate: f32,
) -> Option<Op<T>>
where
    T: Clone + PartialEq + Default,
{
    match node.value() {
        Op::PGM(name, arity, programs, eval_fn) => {
            let new_programs = programs
                .iter()
                .map(|program| {
                    let mut new_program = program.clone();
                    let rand_node = random_provider::range(1..new_program.size());
                    let target_node = new_program.get_mut(rand_node).unwrap();
                    // mutate_tree_node(target_node, store, replace_rate);
                    new_program
                })
                .collect::<Vec<TreeNode<Op<T>>>>();

            Some(Op::PGM(name, *arity, Arc::new(new_programs), *eval_fn))
        }
        _ => None,
    }
}

// fn mutate_tree_node<T>(
//     node: &mut TreeNode<Op<T>>,
//     store: &NodeStore<Op<T>>,
//     replace_rate: f32,
// ) -> usize
// where
//     T: Clone + PartialEq + Default,
// {
//     let mut count = 0;

//     match node.allele() {
//         Op::MutableConst {
//             name,
//             arity,
//             value,
//             supplier: get_value,
//             modifier,
//             operation,
//         } => {
//             let new_value = get_value();
//             let new_value = if random_provider::random::<f32>() < replace_rate {
//                 new_value
//             } else {
//                 modifier(value)
//             };

//             (*node.value_mut()) = Op::MutableConst {
//                 name,
//                 arity: *arity,
//                 value: new_value,
//                 modifier: *modifier,
//                 supplier: *get_value,
//                 operation: *operation,
//             };

//             count += 1;
//         }
//         #[cfg(feature = "pgm")]
//         Op::PGM(name, arity, programs, eval_fn) => {
//             let new_programs = programs
//                 .iter()
//                 .map(|program| {
//                     if random_provider::random::<f32>() < replace_rate {
//                         let depth = program.height().max(3).min(5);
//                         return Tree::with_depth(depth, store.clone())
//                             .take_root()
//                             .unwrap_or_else(|| program.clone());
//                     }
//                     let mut new_program = program.clone();
//                     mutate_tree_node(&mut new_program, store, replace_rate);
//                     new_program
//                 })
//                 .collect::<Vec<TreeNode<Op<T>>>>();

//             (*node.value_mut()) = Op::PGM(name, *arity, Arc::new(new_programs), *eval_fn);

//             count += 1;
//         }
//         _ => {
//             let new_op: Op<T> = store.new_instance(node.node_type());

//             if new_op.arity() == node.arity() {
//                 *node.value_mut() = new_op;
//                 count += 1;
//             }
//         }
//     }

//     count
// }

// // for i in mutation_indexes.iter() {
// //     let current_node = chromosome.get(*i);

// //     if current_node.node_type() == NodeType::Input
// //         || current_node.node_type() == NodeType::Output
// //     {
// //         continue;
// //     }

// //     match current_node.allele() {
// //         Op::MutableConst {
// //             name,
// //             arity,
// //             value,
// //             supplier: get_value,
// //             modifier,
// //             operation,
// //         } => {
// //             let new_value = get_value();
// //             let new_value = if random_provider::random::<f32>() < self.replace_rate {
// //                 new_value
// //             } else {
// //                 modifier(value)
// //             };

// //             (*chromosome.as_mut()[*i].value_mut()) = Op::MutableConst {
// //                 name,
// //                 arity: *arity,
// //                 value: new_value,
// //                 modifier: *modifier,
// //                 supplier: *get_value,
// //                 operation: *operation,
// //             };
// //         }
// //         #[cfg(feature = "pgm")]
// //         Op::PGM(name, arity, programs, eval_fn) => {
// //             let new_programs = programs
// //                 .iter()
// //                 .map(|program| {
// //                     let mut new_program = program.clone();
// //                     mutate_tree_node(
// //                         &mut new_program,
// //                         chromosome.store().unwrap(),
// //                         self.replace_rate,
// //                     );
// //                     new_program
// //                 })
// //                 .collect::<Vec<TreeNode<Op<T>>>>();

// //             (*chromosome.as_mut()[*i].value_mut()) =
// //                 Op::PGM(name, *arity, Arc::new(new_programs), *eval_fn);
// //         }
// //         _ => {
// //             let new_op: Option<Op<T>> = chromosome
// //                 .store()
// //                 .map(|store| store.new_instance(current_node.node_type()));

// //             if let Some(new_op) = new_op {
// //                 if new_op.arity() == current_node.arity() {
// //                     (*chromosome.as_mut()[*i].value_mut()) = new_op;
// //                 }
// //             }
// //         }
// //     }
// // }

// // // count.into()
// // mutation_indexes.len().into()

// // let mut count = 0;
// // for idx in mutation_indexes.iter() {
// //     let current_node = chromosome.get(*idx);

// //     if current_node.node_type() == NodeType::Input
// //         || current_node.node_type() == NodeType::Output
// //     {
// //         continue;
// //     }

// //     match chromosome.store() {
// //         Some(s) => {
// //             let new_op = mutate_node(current_node, &mut count, &s, self.replace_rate);
// //             if let Some(new_op) = new_op {
// //                 (*chromosome.as_mut()[*idx].value_mut()) = new_op;
// //             }
// //         }
// //         None => continue,
// //     };
// // }

// // count.into()

// // // count.into()
// // if let Some(store) = chromosome.store().map(|store| store.clone()) {
// //     for idx in mutation_indexes.iter() {
// //         let current_node = chromosome.get_mut(*idx);
// //         let new_op = mutate_node(current_node, &store, &mut count, self.replace_rate);

// //         if let Some(new_op) = new_op {
// //             (*chromosome.as_mut()[*idx].value_mut()) = new_op;
// //         }
// //     }
// // }
