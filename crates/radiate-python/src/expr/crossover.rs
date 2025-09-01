use radiate::random_provider;

use crate::{CrossoverExpr, Expr, ExprNode, ExprValue};

// impl CrossoverExpr {
//     pub fn apply_crossover<'a, T: ExprNode>(&self, input: ExprValue<'a, T>) -> usize {
//         let mut changed = 0;
//         println!("Applying crossover: {:?}", self);
//         match self {
//             CrossoverExpr::OnePoint => {
//                 let (chrom_one, chrom_two) = match input {
//                     ExprValue::SequencePair(a, b) => (a, b),
//                     _ => return 0,
//                 };

//                 let len1 = chrom_one.len();
//                 let len2 = chrom_two.len();
//                 let n = len1.min(len2);
//                 if n == 0 {
//                     return 0;
//                 }

//                 let cut = if n == 1 {
//                     0
//                 } else {
//                     random_provider::range(0..n)
//                 };
//                 let tail_len = n - cut;

//                 let (_, tail1) = chrom_one.split_at_mut(cut);
//                 let (_, tail2) = chrom_two.split_at_mut(cut);

//                 let tail1 = &mut tail1[..tail_len];
//                 let tail2 = &mut tail2[..tail_len];

//                 tail1.swap_with_slice(tail2);
//                 changed += tail1.len() + tail2.len();
//             }
//             CrossoverExpr::TwoPoint => {
//                 // implement two-point crossover
//             }
//             CrossoverExpr::Swap => {
//                 // implement swap crossover
//             }
//             CrossoverExpr::Mean => {
//                 // implement mean crossover
//             }
//         }
//         changed
//     }
// }
