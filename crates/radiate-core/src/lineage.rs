use crate::phenotype::{FamilyId, PhenotypeId};
use std::collections::HashMap;
// use std::collections::HashSet;
// use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum LineageUpdate {
    Mutate {
        family: FamilyId,
        parent: PhenotypeId,
        child: PhenotypeId,
    },
    Crossover {
        families: (FamilyId, FamilyId),
        parents: (PhenotypeId, PhenotypeId),
        child: PhenotypeId,
    },
    Replace {
        reason: &'static str,
        old: (FamilyId, PhenotypeId),
        new: (FamilyId, PhenotypeId),
    },
    Invalid,
}

impl
    From<(
        (FamilyId, FamilyId),
        (PhenotypeId, PhenotypeId),
        PhenotypeId,
    )> for LineageUpdate
{
    fn from(
        (parent_lineages, parent_versions, child_id): (
            (FamilyId, FamilyId),
            (PhenotypeId, PhenotypeId),
            PhenotypeId,
        ),
    ) -> Self {
        if parent_versions.0 == child_id || parent_versions.1 == child_id {
            return LineageUpdate::Invalid;
        }

        LineageUpdate::Crossover {
            families: parent_lineages,
            parents: parent_versions,
            child: child_id,
        }
    }
}

impl From<((FamilyId, PhenotypeId), PhenotypeId)> for LineageUpdate {
    fn from((parent_id, child_id): ((FamilyId, PhenotypeId), PhenotypeId)) -> Self {
        if parent_id.1 == child_id {
            return LineageUpdate::Invalid;
        }

        LineageUpdate::Mutate {
            family: parent_id.0,
            parent: parent_id.1,
            child: child_id,
        }
    }
}

impl
    From<(
        &'static str,
        (FamilyId, PhenotypeId),
        (FamilyId, PhenotypeId),
    )> for LineageUpdate
{
    fn from(
        (reason, (old_family, old_id), (new_family, new_id)): (
            &'static str,
            (FamilyId, PhenotypeId),
            (FamilyId, PhenotypeId),
        ),
    ) -> Self {
        if old_id == new_id {
            return LineageUpdate::Invalid;
        }

        LineageUpdate::Replace {
            reason,
            old: (old_family, old_id),
            new: (new_family, new_id),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LineageEvent {
    Mutate {
        operator: &'static str,
        family: FamilyId,
        parent: PhenotypeId,
        child: PhenotypeId,
    },
    Crossover {
        operator: &'static str,
        families: (FamilyId, FamilyId),
        parents: (PhenotypeId, PhenotypeId),
        child: PhenotypeId,
    },
    Replace {
        reason: &'static str,
        old: (FamilyId, PhenotypeId),
        new: (FamilyId, PhenotypeId),
    },
}

#[derive(Clone, Debug, Default)]
pub struct LineageStats {
    pub updates: usize,
    pub parent_usage: HashMap<PhenotypeId, usize>,
    pub family_usage: HashMap<FamilyId, usize>,
    pub family_pairs: HashMap<(FamilyId, FamilyId), usize>,
    pub cross_family_crossovers: usize,
    pub within_family_crossovers: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Lineage {
    stats: LineageStats,
    // ancestory: VecDeque<HashMap<PhenotypeId, AncestorNode>>,
}

impl Lineage {
    pub fn rollover(&mut self) {
        self.stats = LineageStats::default();
    }

    pub fn stats(&self) -> &LineageStats {
        &self.stats
    }

    pub fn extend<I: IntoIterator<Item = impl Into<LineageUpdate>>>(
        &mut self,
        operation: &'static str,
        events: I,
    ) {
        for event in events {
            self.push(operation, event.into());
        }
    }

    pub fn push(&mut self, operator: &'static str, update: LineageUpdate) {
        match update {
            LineageUpdate::Invalid => return,
            LineageUpdate::Mutate {
                family,
                parent,
                child,
            } => {
                *self.stats.parent_usage.entry(parent).or_insert(0) += 1;
                *self.stats.family_usage.entry(family).or_insert(0) += 1;

                LineageEvent::Mutate {
                    operator,
                    child,
                    family,
                    parent,
                }
            }
            LineageUpdate::Crossover {
                families,
                parents,
                child,
            } => {
                *self.stats.parent_usage.entry(parents.0).or_insert(0) += 1;
                *self.stats.parent_usage.entry(parents.1).or_insert(0) += 1;
                *self.stats.family_usage.entry(families.0).or_insert(0) += 1;
                *self.stats.family_usage.entry(families.1).or_insert(0) += 1;

                if families.0 == families.1 {
                    self.stats.within_family_crossovers += 1;
                } else {
                    self.stats.cross_family_crossovers += 1;
                };

                let (a, b) = if families.0 <= families.1 {
                    (families.0, families.1)
                } else {
                    (families.1, families.0)
                };

                *self.stats.family_pairs.entry((a, b)).or_insert(0) += 1;

                LineageEvent::Crossover {
                    operator,
                    families,
                    parents,
                    child,
                }
            }
            LineageUpdate::Replace { reason, old, new } => {
                LineageEvent::Replace { reason, old, new }
            }
        };

        self.stats.updates += 1;
    }
}

// #[derive(Clone, Debug)]
// struct AncestorNode {
//     family: FamilyId,
//     parents: Parents,
// }

// #[derive(Clone, Debug)]
// enum Parents {
//     None,
//     One(PhenotypeId),
//     Two(PhenotypeId, PhenotypeId),
// }
