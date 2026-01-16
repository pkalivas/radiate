use crate::factor::DiscreteFactor;
use crate::var::VarSpec;

pub trait FactorKernel {
    fn param_count(&self) -> usize;
    fn build(&self, params: &[f32]) -> Result<DiscreteFactor, String>;
}

/// CPT kernel: builds P(child | parents...) as a discrete table with scope [parents..., child]
/// Params are logits in row-major over that scope.
pub struct CptKernel {
    pub parents: Vec<VarSpec>,
    pub child: VarSpec,
}

impl FactorKernel for CptKernel {
    fn param_count(&self) -> usize {
        let mut dims = self.parents.iter().map(|v| v.card).collect::<Vec<usize>>();
        dims.push(self.child.card);
        crate::prod_usize(&dims)
    }

    fn build(&self, params: &[f32]) -> Result<DiscreteFactor, String> {
        if params.len() != self.param_count() {
            return Err(format!(
                "params len {} != {}",
                params.len(),
                self.param_count()
            ));
        }
        let mut scope = self.parents.clone();
        scope.push(self.child);

        let mut f = DiscreteFactor::new(scope, params.to_vec())?;
        f.normalize_rows(self.child.id)?;

        Ok(f)
    }
}

/// Ising pairwise kernel over two binary vars: scope [a,b], card 2 each.
/// Params: [h_a0, h_a1, h_b0, h_b1, J_same, J_diff] (simple template)
pub struct IsingKernel {
    pub a: VarSpec,
    pub b: VarSpec,
}

impl FactorKernel for IsingKernel {
    fn param_count(&self) -> usize {
        6
    }

    fn build(&self, params: &[f32]) -> Result<DiscreteFactor, String> {
        if self.a.card != 2 || self.b.card != 2 {
            return Err("IsingKernel expects binary vars".into());
        }

        if params.len() != 6 {
            return Err("IsingKernel expects 6 params".into());
        }

        let ha0 = params[0];
        let ha1 = params[1];
        let hb0 = params[2];
        let hb1 = params[3];
        let j_same = params[4];
        let j_diff = params[5];

        // logp(a,b) = h_a[a] + h_b[b] + (a==b ? j_same : j_diff)
        // axis order [a,b], row-major => idx = a + 2*b? (with strides [1,2]) => idx = a + b*2
        let mut logp = vec![0.0f32; 4];
        for a in 0..2u32 {
            for b in 0..2u32 {
                let ha = if a == 0 { ha0 } else { ha1 };
                let hb = if b == 0 { hb0 } else { hb1 };
                let j = if a == b { j_same } else { j_diff };
                let idx = (a as usize) + (b as usize) * 2;
                logp[idx] = ha + hb + j;
            }
        }

        DiscreteFactor::new(vec![self.a, self.b], logp)
    }
}
