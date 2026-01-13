use crate::NodeStore;
use crate::{NodeType, Op};
use std::collections::BTreeMap;

impl NodeStore<Op<f32>> {
    /// Build a dense vector `var_cards[idx] = Some(k)` from Input ops.
    /// If a variable has no domain (None) we store None (you can default later).
    pub fn var_cards(&self) -> Vec<Option<usize>> {
        // Collect idx -> card from Input
        self.map_by_type(NodeType::Input, |vals| {
            vals.iter()
                .filter_map(|nv| match nv.value() {
                    Op::Var(_, idx, card) => Some((*idx, card.clone())),
                    _ => None,
                })
                .collect::<BTreeMap<usize, Option<usize>>>()
        })
        .map(|card_map| {
            if card_map.is_empty() {
                return vec![];
            }

            let max_idx = *card_map.keys().next_back().unwrap();
            let mut out = vec![None; max_idx + 1];
            for (idx, card) in card_map {
                out[idx] = card;
            }

            out
        })
        .unwrap_or_default()
    }

    /// For a given scope (var indices), return the Shape dims in the *same order*.
    /// `None` domains become 1 (or choose your default).
    pub fn cards_for_scope(&self, scope: &[usize]) -> Vec<usize> {
        let cards = self.var_cards();
        scope
            .iter()
            .map(|&v| cards.get(v).and_then(|x| *x).unwrap_or(1))
            .collect()
    }
}
