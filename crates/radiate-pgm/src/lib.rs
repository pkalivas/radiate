use radiate_core::{
    BatchFitnessFunction, Chromosome, Codec, Gene, Genotype, Valid, fitness::FitnessFunction,
    random_provider,
};
use radiate_utils::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarSpec {
    pub id: usize,
    pub card: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactorKind {
    /// Discrete conditional table: log P(child | parents)
    /// Scope order is [parents..., child]
    Logp,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FactorGene {
    /// ordered scope: [parents..., child]
    pub scope: Vec<usize>,
    pub kind: FactorKind,
    /// cached table shape (for Logp: [cards(parents)..., card(child)])
    pub shape: Vec<usize>,
    /// parameters (for Logp: Value::Array table with shape [cards(parents)..., card(child)])
    pub params: Value<f32>,
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
            FactorKind::Logp => init_logp_table(&self.shape),
        };

        FactorGene {
            scope: self.scope.clone(),
            kind: self.kind.clone(),
            shape: self.shape.clone(),
            params,
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        allele.clone()
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
    pub vars: Vec<VarSpec>,
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
        // basic checks: each factor's scope vars exist
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

fn sample_scope(num_vars: usize, max_scope: usize) -> Vec<usize> {
    let k = random_provider::range(1..max_scope.min(num_vars).max(1) + 1);

    // sample without replacement
    let mut picked = Vec::with_capacity(k);
    while picked.len() < k {
        let v = random_provider::range(0..num_vars);
        if !picked.contains(&v) {
            picked.push(v);
        }
    }

    // enforce "child last" convention for Logp when len>1
    if picked.len() > 1 {
        let child_pos = random_provider::range(0..picked.len());
        let child = picked.remove(child_pos);
        picked.push(child);
    }

    picked
}

fn logp_table_shape(vars: &[VarSpec], scope: &[usize]) -> Vec<usize> {
    scope.iter().map(|&vid| vars[vid].card.max(1)).collect()
}

fn init_logp_table(shape: &[usize]) -> Value<f32> {
    // IMPORTANT: store *logits* and interpret via log-softmax at eval time (like your Op::logprob_table)
    // That makes mutation smooth.
    Value::from((shape.to_vec(), |_| random_provider::range(-1.0..1.0)))
}

#[derive(Clone)]
pub struct PgmCodec {
    pub vars: Vec<VarSpec>,
    pub num_factors: usize,
    pub max_scope: usize,
}

impl PgmCodec {
    pub fn new(cards: &[usize], num_factors: usize, max_scope: usize) -> Self {
        let vars = cards
            .iter()
            .enumerate()
            .map(|(id, &card)| VarSpec { id, card })
            .collect();

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
            let scope = sample_scope(num_vars, self.max_scope);
            let shape = logp_table_shape(&self.vars, &scope);
            let params = init_logp_table(&shape);

            factors.push(FactorGene {
                scope,
                kind: FactorKind::Logp,
                shape: shape.clone(),
                params,
            });
        }

        let chrom = PgmChromosome {
            vars: self.vars.clone(),
            factors,
        };

        Genotype::from(vec![chrom])
    }

    fn decode(&self, genotype: &Genotype<PgmChromosome>) -> PgmChromosome {
        genotype[0].clone()
    }
}

#[derive(Clone)]
pub struct PgmDataSet {
    pub rows: Vec<Vec<Option<usize>>>,
}

impl PgmDataSet {
    pub fn new(rows: Vec<Vec<Option<usize>>>) -> Self {
        Self { rows }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Clone)]
pub struct PgmLogLik {
    data: PgmDataSet,
}

impl PgmLogLik {
    pub fn new(data: PgmDataSet) -> Self {
        Self { data }
    }

    fn eval_logp_factor(
        &self,
        chrom: &PgmChromosome,
        factor: &FactorGene,
        row: &[Option<usize>],
    ) -> f32 {
        match factor.kind {
            FactorKind::Logp => {
                let mut idxs = Vec::with_capacity(factor.scope.len());
                for &vid in &factor.scope {
                    let Some(s) = row[vid] else {
                        return 0.0;
                    };
                    idxs.push(s.min(chrom.vars[vid].card.saturating_sub(1)));
                }

                // Compute log-softmax at this parent config, exactly like  Op::logprob_table.
                // params is Value::Array with shape [cards(parents)..., child_card]
                logprob_table_eval(&factor.params, &idxs)
            }
        }
    }

    pub fn loglik(&self, chrom: &PgmChromosome) -> f32 {
        let mut ll = 0.0;
        for row in &self.data.rows {
            for f in &chrom.factors {
                ll += self.eval_logp_factor(chrom, f, row);
            }
        }
        ll
    }

    pub fn neg_mean_loglik(&self, chrom: &PgmChromosome) -> f32 {
        let n = self.data.len().max(1) as f32;
        // minimize negative average log-likelihood
        -(self.loglik(chrom) / n)
    }
}

fn logprob_table_eval(val: &Value<f32>, idxs: &[usize]) -> f32 {
    let Value::Array {
        values,
        shape,
        strides,
    } = val
    else {
        return 0.0;
    };

    let rank = shape.rank();
    if idxs.len() != rank {
        return 0.0;
    }
    if rank == 0 {
        return 0.0;
    }

    let child_axis = rank - 1;
    let child = idxs[child_axis];
    let child_states = shape.dim_at(child_axis).max(1);

    // base offset with child fixed to 0
    let mut base = 0usize;
    for i in 0..child_axis {
        let dim = shape.dim_at(i).max(1);
        let idx = idxs[i].min(dim - 1);
        base = base.saturating_add(idx.saturating_mul(strides.stride_at(i)));
    }

    // max logit
    let mut max_logit = f32::NEG_INFINITY;
    for k in 0..child_states {
        let pos = base.saturating_add(k.saturating_mul(strides.stride_at(child_axis)));
        let s = values.get(pos).copied().unwrap_or(0.0);
        max_logit = max_logit.max(s);
    }
    if !max_logit.is_finite() {
        return 0.0;
    }

    // logsumexp
    let mut sum_exp = 0.0f32;
    for k in 0..child_states {
        let pos = base.saturating_add(k.saturating_mul(strides.stride_at(child_axis)));
        let s = values.get(pos).copied().unwrap_or(0.0);
        sum_exp += (s - max_logit).exp();
    }
    let lse = max_logit + sum_exp.ln();

    let child = child.min(child_states.saturating_sub(1));
    let child_pos = base.saturating_add(child.saturating_mul(strides.stride_at(child_axis)));
    let child_logit = values.get(child_pos).copied().unwrap_or(0.0);

    child_logit - lse
}

impl FitnessFunction<PgmChromosome, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, input: PgmChromosome) -> f32 {
        self.neg_mean_loglik(&input)
    }
}

impl<'a> FitnessFunction<&'a Genotype<PgmChromosome>, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, input: &'a Genotype<PgmChromosome>) -> f32 {
        self.neg_mean_loglik(&input[0])
    }
}

impl BatchFitnessFunction<PgmChromosome, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, inputs: Vec<PgmChromosome>) -> Vec<f32> {
        inputs
            .into_iter()
            .map(|c| self.neg_mean_loglik(&c))
            .collect()
    }
}

impl<'a> BatchFitnessFunction<&'a Genotype<PgmChromosome>, f32> for PgmLogLik {
    #[inline]
    fn evaluate(&self, inputs: Vec<&'a Genotype<PgmChromosome>>) -> Vec<f32> {
        inputs
            .into_iter()
            .map(|g| self.neg_mean_loglik(&g[0]))
            .collect()
    }
}
