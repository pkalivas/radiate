use std::sync::Arc;

use radiate_core::{Chromosome, Gene, Valid};
use radiate_utils::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarSpec {
    pub id: usize,
    pub card: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactorKind {
    Logp,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FactorGene {
    pub scope: Vec<usize>,
    pub kind: FactorKind,
    pub shape: Vec<usize>,
    pub params: Value<f32>,
}

impl FactorGene {
    #[inline]
    pub fn resample_scope(&mut self, vars: &[VarSpec], max_scope: usize) {
        let scope = super::sample_scope(vars.len(), max_scope);
        let shape = super::logp_table_shape(vars, &scope);
        let params = super::init_logp_table(&shape);

        self.scope = scope;
        self.shape = shape;
        self.params = params;
    }
}

impl Gene for FactorGene {
    type Allele = Self;

    fn allele(&self) -> &Self::Allele {
        self
    }

    fn allele_mut(&mut self) -> &mut Self::Allele {
        self
    }

    fn new_instance(&self) -> Self {
        let params = match self.kind {
            FactorKind::Logp => super::init_logp_table(&self.shape),
        };

        FactorGene {
            scope: self.scope.clone(),
            kind: self.kind.clone(),
            shape: self.shape.clone(),
            params,
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        FactorGene {
            scope: allele.scope.clone(),
            kind: allele.kind.clone(),
            shape: allele.shape.clone(),
            params: allele.params.clone(),
        }
    }
}

impl Valid for FactorGene {
    fn is_valid(&self) -> bool {
        !self.scope.is_empty()
            && self.shape.len() == self.scope.len()
            && self.shape.iter().all(|&d| d >= 1)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PgmChromosome {
    pub vars: Arc<[VarSpec]>,
    pub factors: Vec<FactorGene>,
}

impl Chromosome for PgmChromosome {
    type Gene = FactorGene;

    fn genes(&self) -> &[FactorGene] {
        &self.factors
    }

    fn genes_mut(&mut self) -> &mut [FactorGene] {
        &mut self.factors
    }
}

impl Valid for PgmChromosome {
    fn is_valid(&self) -> bool {
        let num_vars = self.vars.len();
        for factor in &self.factors {
            for &vid in &factor.scope {
                if vid >= num_vars {
                    return false;
                }
            }

            if factor.shape.len() != factor.scope.len() {
                return false;
            }

            for (i, &vid) in factor.scope.iter().enumerate() {
                let expected = self.vars[vid].card.max(1);
                if factor.shape[i].max(1) != expected {
                    return false;
                }
            }
        }

        true
    }
}
