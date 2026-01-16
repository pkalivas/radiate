use crate::{FactorGene, FactorKind, PgmChromosome, VarId, VarSpec};
use radiate_core::{Codec, Genotype};
use std::sync::Arc;

#[derive(Clone)]
pub struct PgmCodec {
    pub vars: Arc<[VarSpec]>,
    pub num_factors: usize,
    pub max_scope: usize,
}

impl PgmCodec {
    pub fn new(cards: &[usize], num_factors: usize, max_scope: usize) -> Self {
        let vars = cards
            .iter()
            .enumerate()
            .map(|(id, &card)| VarSpec {
                id: VarId(id as u32),
                card,
            })
            .collect::<Arc<[VarSpec]>>();

        Self {
            vars,
            num_factors,
            max_scope,
        }
    }
}

impl Codec<PgmChromosome, PgmChromosome> for PgmCodec {
    fn encode(&self) -> Genotype<PgmChromosome> {
        let num_vars = self.vars.len();

        let mut factors = Vec::with_capacity(self.num_factors);
        for _ in 0..self.num_factors {
            let scope = super::sample_scope(num_vars, self.max_scope);
            let shape = super::logp_table_shape(&self.vars, &scope);
            let params = super::init_logp_table(&shape);

            println!("factor scope: {:?}", params);

            factors.push(FactorGene {
                scope,
                kind: FactorKind::Logp,
                shape: shape.clone(),
                params,
            });
        }

        Genotype::from(PgmChromosome {
            vars: self.vars.clone(),
            factors,
        })
    }

    fn decode(&self, genotype: &Genotype<PgmChromosome>) -> PgmChromosome {
        genotype[0].clone()
    }
}
