// use crate::phenotype::PhenotypeId;
// use std::collections::{HashMap, HashSet};

// #[derive(Clone, Debug, Default)]
// pub struct LineageStats {
//     pub updates: usize,
//     pub parent_usage: HashMap<PhenotypeId, usize>,
//     pub parent_mapping: HashMap<PhenotypeId, Vec<PhenotypeId>>,
//     pub cross_family_crossovers: usize,
//     pub within_family_crossovers: usize,
// }

// impl LineageStats {
//     pub fn clear(&mut self) {
//         self.updates = 0;
//         self.parent_usage.clear();
//         self.parent_mapping.clear();
//         self.cross_family_crossovers = 0;
//         self.within_family_crossovers = 0;
//     }
// }

// #[derive(Clone, Debug, Default)]
// pub struct Lineage {
//     stats: [LineageStats; 5],
// }

// impl Lineage {
//     pub fn rollover(&mut self) {
//         self.stats.rotate_right(1);
//         self.stats[0].clear();
//     }

//     pub fn stats(&self) -> &LineageStats {
//         &self.stats[0]
//     }

//     pub fn extend<I: IntoIterator<Item = impl Into<LineageUpdate>>>(&mut self, events: I) {
//         for event in events {
//             self.push(event.into());
//         }
//     }

//     pub fn get_ancestors(&self, phenotype_id: PhenotypeId) -> Vec<HashSet<PhenotypeId>> {
//         let mut ancestors = Vec::new();
//         let mut stack = vec![phenotype_id];

//         for i in 0..self.stats.len() {
//             let mut next_level = HashSet::new();
//             for id in stack {
//                 if let Some(parents) = self.stats[i].parent_mapping.get(&id) {
//                     for parent in parents {
//                         next_level.insert(*parent);
//                     }
//                 }
//             }

//             if next_level.is_empty() {
//                 break;
//             }

//             ancestors.push(next_level.clone());
//             stack = next_level.into_iter().collect();
//         }

//         ancestors
//     }

//     pub fn push(&mut self, update: LineageUpdate) {
//         match update {
//             LineageUpdate::Invalid => return,
//             LineageUpdate::Mutate {
//                 // family,
//                 parent,
//                 child,
//             } => {
//                 *self.stats[0].parent_usage.entry(parent).or_insert(0) += 1;

//                 self.stats[0]
//                     .parent_mapping
//                     .entry(child)
//                     .or_insert_with(Vec::new)
//                     .push(parent);
//             }
//             LineageUpdate::Crossover {
//                 // families,
//                 parents,
//                 child,
//             } => {
//                 self.stats[0]
//                     .parent_mapping
//                     .entry(child)
//                     .or_insert_with(Vec::new)
//                     .push(parents.0);
//                 self.stats[0]
//                     .parent_mapping
//                     .entry(child)
//                     .or_insert_with(Vec::new)
//                     .push(parents.1);

//                 *self.stats[0].parent_usage.entry(parents.0).or_insert(0) += 1;
//                 *self.stats[0].parent_usage.entry(parents.1).or_insert(0) += 1;
//             }
//         };

//         self.stats[0].updates += 1;
//     }
// }

// #[derive(Clone, Debug)]
// pub enum LineageUpdate {
//     Mutate {
//         parent: PhenotypeId,
//         child: PhenotypeId,
//     },
//     Crossover {
//         parents: (PhenotypeId, PhenotypeId),
//         child: PhenotypeId,
//     },
//     Invalid,
// }

// impl
//     From<(
//         // (FamilyId, FamilyId),
//         (PhenotypeId, PhenotypeId),
//         PhenotypeId,
//     )> for LineageUpdate
// {
//     fn from(
//         (parent_versions, child_id): (
//             // (FamilyId, FamilyId),
//             (PhenotypeId, PhenotypeId),
//             PhenotypeId,
//         ),
//     ) -> Self {
//         if parent_versions.0 == child_id || parent_versions.1 == child_id {
//             return LineageUpdate::Invalid;
//         }

//         LineageUpdate::Crossover {
//             // families: parent_lineages,
//             parents: parent_versions,
//             child: child_id,
//         }
//     }
// }

// impl From<(PhenotypeId, PhenotypeId)> for LineageUpdate {
//     fn from((parent_id, child_id): (PhenotypeId, PhenotypeId)) -> Self {
//         if parent_id == child_id {
//             return LineageUpdate::Invalid;
//         }

//         LineageUpdate::Mutate {
//             parent: parent_id,
//             child: child_id,
//         }
//     }
// }
