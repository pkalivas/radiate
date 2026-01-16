use crate::var::{VarId, VarSpec};
use crate::{log_normalize_in_place, logsumexp, prod_usize};

#[derive(Clone, Debug)]
pub struct DiscreteFactor {
    scope: Vec<VarId>,   // axis order
    dims: Vec<usize>,    // cardinalities by axis
    strides: Vec<usize>, // row-major strides
    logp: Vec<f32>,      // contiguous log-table
}

impl DiscreteFactor {
    /// Build from explicit scope specs (order is preserved).
    pub fn new(scope: Vec<VarSpec>, logp: Vec<f32>) -> Result<Self, String> {
        let mut svars = Vec::with_capacity(scope.len());
        let mut dims = Vec::with_capacity(scope.len());
        for v in scope {
            svars.push(v.id);
            dims.push(v.card);
        }

        let strides = Self::compute_strides(&dims);
        let expected = prod_usize(&dims);

        if logp.len() != expected {
            return Err(format!(
                "logp length {} != expected {}",
                logp.len(),
                expected
            ));
        }

        Ok(Self {
            scope: svars,
            dims,
            strides,
            logp,
        })
    }

    /// Convenient: all zeros in prob-space => log(1) table (not normalized unless you normalize).
    pub fn uniform(scope: Vec<VarSpec>) -> Result<Self, String> {
        let dims: Vec<usize> = scope.iter().map(|v| v.card).collect();
        let n = prod_usize(&dims);
        Self::new(scope, vec![0.0; n])
    }

    pub fn scope(&self) -> &[VarId] {
        &self.scope
    }

    pub fn dims(&self) -> &[usize] {
        &self.dims
    }

    pub fn logp(&self) -> &[f32] {
        &self.logp
    }

    #[inline]
    fn compute_strides(dims: &[usize]) -> Vec<usize> {
        let mut strides = vec![1usize; dims.len()];
        let mut acc = 1usize;
        for (i, &d) in dims.iter().enumerate() {
            strides[i] = acc;
            acc = acc.saturating_mul(d);
        }
        strides
    }

    /// Flatten an assignment (aligned with this factor's axis order) to an index.
    #[inline]
    pub fn index_of(&self, asg: &[usize]) -> usize {
        debug_assert_eq!(asg.len(), self.scope.len());
        let mut idx = 0usize;
        for i in 0..asg.len() {
            idx += (asg[i] as usize) * self.strides[i];
        }
        idx
    }

    /// Unflatten index -> assignment (aligned with axis order).
    pub fn unflatten(&self, idx: usize) -> Vec<usize> {
        let mut asg = vec![0usize; self.scope.len()];
        for i in (0..self.scope.len()).rev() {
            let d = self.dims[i] as usize;
            let v = (idx / self.strides[i]) % d;
            asg[i] = v as usize;
        }
        asg
    }

    #[inline]
    pub fn log_value_aligned(&self, asg: &[usize]) -> f32 {
        let idx = self.index_of(asg);
        self.logp[idx]
    }

    /// Restrict (condition) the factor on evidence.
    /// Evidence is a list of (VarId, state) pairs.
    /// Returns a new factor with those variables fixed and removed from the scope.
    pub fn restrict(&self, evidence: &[(VarId, usize)]) -> Result<Self, String> {
        // Separate variables into fixed (evidence) and remaining (keep)
        let mut keep = Vec::new();
        for &v in &self.scope {
            if !evidence.iter().any(|(ev, _)| *ev == v) {
                keep.push(v);
            }
        }

        // Build the new factor scope with only the kept variables
        let keep_specs = keep
            .iter()
            .map(|&v| {
                let ax = self.axis_of(v).unwrap();
                VarSpec {
                    id: v,
                    card: self.dims[ax],
                }
            })
            .collect::<Vec<_>>();

        let mut out = DiscreteFactor::uniform(keep_specs)?;
        out.logp.fill(f32::NEG_INFINITY);

        // Build a base assignment with evidence values fixed
        let mut base_asg = vec![0usize; self.scope.len()];
        for (ev_var, ev_val) in evidence {
            if let Some(ax) = self.axis_of(*ev_var) {
                base_asg[ax] = *ev_val;
            }
        }

        // For each assignment of the kept variables, copy the value from self
        let out_len = out.logp.len();
        for out_idx in 0..out_len {
            let out_asg = out.unflatten(out_idx);

            // Fill in the kept variable assignments
            for (k, &v) in keep.iter().enumerate() {
                let ax = self.axis_of(v).unwrap();
                base_asg[ax] = out_asg[k];
            }

            out.logp[out_idx] = self.log_value_aligned(&base_asg);
        }

        Ok(out)
    }

    /// Reorder axes to `new_scope` (must be a permutation of current scope).
    pub fn reorder(&self, new_scope: &[VarId]) -> Result<Self, String> {
        if new_scope.len() != self.scope.len() {
            return Err("new_scope length mismatch".into());
        }
        // map var -> old axis
        let mut old_pos = std::collections::BTreeMap::<VarId, usize>::new();
        for (i, &v) in self.scope.iter().enumerate() {
            old_pos.insert(v, i);
        }
        let mut perm = Vec::with_capacity(new_scope.len());
        let mut new_dims = Vec::with_capacity(new_scope.len());
        for &v in new_scope {
            let &p = old_pos
                .get(&v)
                .ok_or_else(|| "new_scope is not a permutation".to_string())?;
            perm.push(p);
            new_dims.push(self.dims[p]);
        }

        let new_strides = Self::compute_strides(&new_dims);
        let new_len = prod_usize(&new_dims);
        let mut new_logp = vec![f32::NEG_INFINITY; new_len];

        // For each assignment in new space, map to old assignment and copy value.
        for new_idx in 0..new_len {
            // compute new assignment
            let mut new_asg = vec![0usize; new_scope.len()];
            for i in (0..new_scope.len()).rev() {
                let d = new_dims[i] as usize;
                let v = (new_idx / new_strides[i]) % d;
                new_asg[i] = v as usize;
            }
            // old assignment in old axis order
            let mut old_asg = vec![0usize; self.scope.len()];
            for (new_axis, &old_axis) in perm.iter().enumerate() {
                old_asg[old_axis] = new_asg[new_axis];
            }
            new_logp[new_idx] = self.log_value_aligned(&old_asg);
        }

        Ok(Self {
            scope: new_scope.to_vec(),
            dims: new_dims,
            strides: new_strides,
            logp: new_logp,
        })
    }

    /// CPT-style normalization on the `child` axis: for every fixed parent assignment,
    /// normalize over child states so that sum prob == 1.
    pub fn normalize_rows(&mut self, child: VarId) -> Result<(), String> {
        let axis = self
            .axis_of(child)
            .ok_or_else(|| "child not in scope".to_string())?;
        let child_card = self.dims[axis] as usize;

        // We will iterate all "rows" where row = varying child with parents fixed.
        // For row-major strides: indices for a fixed parent assignment are spaced by stride[axis].
        // But other axes also vary; easiest is: enumerate all assignments of non-child axes,
        // then gather child slice.
        let non_axes: Vec<usize> = (0..self.scope.len()).filter(|&i| i != axis).collect();
        let non_dims: Vec<usize> = non_axes.iter().map(|&i| self.dims[i]).collect();
        let non_strides = Self::compute_strides(&non_dims);
        let rows = prod_usize(&non_dims);

        let mut base_asg = vec![0usize; self.scope.len()];

        for row_idx in 0..rows {
            // decode non-child assignment into base_asg
            for (k, &ax) in non_axes.iter().enumerate() {
                let d = non_dims[k] as usize;
                let v = (row_idx / non_strides[k]) % d;
                base_asg[ax] = v;
            }

            // collect row over child
            let mut row = vec![0.0f32; child_card];
            for c in 0..child_card {
                base_asg[axis] = c;
                row[c] = self.log_value_aligned(&base_asg);
            }

            // normalize in log-space
            log_normalize_in_place(&mut row);

            // write back
            for c in 0..child_card {
                base_asg[axis] = c;
                let idx = self.index_of(&base_asg);
                self.logp[idx] = row[c];
            }
        }
        Ok(())
    }

    /// Sum out the given vars.
    pub fn marginalize(&self, elim: &[VarId]) -> Result<Self, String> {
        // keep vars not eliminated
        let mut keep = Vec::new();
        for &v in &self.scope {
            if !elim.contains(&v) {
                keep.push(v);
            }
        }
        // if nothing kept => scalar factor
        let keep_specs = keep
            .iter()
            .map(|&v| {
                let ax = self.axis_of(v).unwrap();
                VarSpec {
                    id: v,
                    card: self.dims[ax],
                }
            })
            .collect::<Vec<_>>();

        let mut out = DiscreteFactor::uniform(keep_specs)?;
        // out.logp will be overwritten by logsumexp accumulations; init -inf
        out.logp.fill(f32::NEG_INFINITY);

        // Map out assignment -> collect all matching self assignments across eliminated vars.
        let out_len = out.logp.len();
        for out_idx in 0..out_len {
            let out_asg = out.unflatten(out_idx);

            // Build a partial assignment in self-space for kept vars.
            let mut base = vec![0usize; self.scope.len()];
            for (k, &v) in keep.iter().enumerate() {
                let ax = self.axis_of(v).unwrap();
                base[ax] = out_asg[k];
            }

            // enumerate eliminated assignments
            let elim_axes: Vec<usize> = self
                .scope
                .iter()
                .enumerate()
                .filter(|(_, v)| elim.contains(v))
                .map(|(i, _)| i)
                .collect();
            let elim_dims: Vec<usize> = elim_axes.iter().map(|&i| self.dims[i]).collect();
            let elim_strides = Self::compute_strides(&elim_dims);
            let elim_len = prod_usize(&elim_dims);

            let mut buf = Vec::with_capacity(elim_len.max(1));
            if elim_axes.is_empty() {
                buf.push(self.log_value_aligned(&base));
            } else {
                for eidx in 0..elim_len {
                    for (k, &ax) in elim_axes.iter().enumerate() {
                        let d = elim_dims[k];
                        let v = (eidx / elim_strides[k]) % d;
                        base[ax] = v;
                    }
                    buf.push(self.log_value_aligned(&base));
                }
            }

            out.logp[out_idx] = logsumexp(&buf);
        }

        Ok(out)
    }

    /// Product of two discrete factors: join scope, broadcast, add in log-space.
    pub fn product(
        &self,
        rhs: &DiscreteFactor,
        cards: &impl Fn(VarId) -> usize,
    ) -> Result<Self, String> {
        // union scope: keep self order then append rhs vars not present
        let mut out_scope = self.scope.clone();
        for &v in rhs.scope.iter() {
            if !out_scope.contains(&v) {
                out_scope.push(v);
            }
        }
        // build VarSpec using provided card lookup
        let out_specs = out_scope
            .iter()
            .map(|&v| VarSpec {
                id: v,
                card: cards(v),
            })
            .collect::<Vec<_>>();
        let mut out = DiscreteFactor::uniform(out_specs)?;
        out.logp.fill(f32::NEG_INFINITY);

        // precompute axis maps
        let self_map = out_scope
            .iter()
            .map(|&v| self.axis_of(v))
            .collect::<Vec<_>>();
        let rhs_map = out_scope
            .iter()
            .map(|&v| rhs.axis_of(v))
            .collect::<Vec<_>>();

        // enumerate out assignments
        let out_len = out.logp.len();
        for out_idx in 0..out_len {
            let out_asg = out.unflatten(out_idx);

            // build aligned assignments for each factor
            let mut asg_a = vec![0usize; self.scope.len()];
            let mut asg_b = vec![0usize; rhs.scope.len()];

            for (out_axis, &val) in out_asg.iter().enumerate() {
                if let Some(ax) = self_map[out_axis] {
                    asg_a[ax] = val;
                }
                if let Some(ax) = rhs_map[out_axis] {
                    asg_b[ax] = val;
                }
            }

            let la = self.log_value_aligned(&asg_a);
            let lb = rhs.log_value_aligned(&asg_b);
            out.logp[out_idx] = la + lb;
        }

        Ok(out)
    }

    #[inline]
    pub fn axis_of(&self, v: VarId) -> Option<usize> {
        self.scope.iter().position(|&x| x == v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::var::VarSpec;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn indexing_roundtrip() {
        let f = DiscreteFactor::uniform(vec![
            VarSpec::new(0, 2),
            VarSpec::new(1, 3),
            VarSpec::new(2, 4),
        ])
        .unwrap();

        for idx in 0..f.logp.len() {
            let asg = f.unflatten(idx);
            let idx2 = f.index_of(&asg);
            assert_eq!(idx, idx2);
        }
    }

    #[test]
    fn reorder_preserves_values() {
        // scope: [A,B], dims [2,3], fill logp = idx as f32
        let scope = vec![VarSpec::new(0, 2), VarSpec::new(1, 3)];
        let mut logp = vec![0.0; 6];
        for i in 0..6 {
            logp[i] = i as f32;
        }
        let f = DiscreteFactor::new(scope, logp).unwrap();

        let a = VarId(0);
        let b = VarId(1);

        let g = f.reorder(&[b, a]).unwrap();

        // For each assignment in g-space, value equals f with swapped assignment.
        for bi in 0..3 {
            for ai in 0..2 {
                let gv = g.log_value_aligned(&[bi, ai]);
                let fv = f.log_value_aligned(&[ai, bi]);
                assert_eq!(gv, fv);
            }
        }
    }

    #[test]
    fn normalize_rows_makes_rows_sum_to_1() {
        // CPT factor P(C | A) with A card 2, C card 3. scope [A,C].
        let scope = vec![VarSpec::new(0, 2), VarSpec::new(1, 3)];
        // logits arbitrary
        let logp = vec![
            0.0, 1.0, 2.0, // A=0, C=0..2
            2.0, 1.0, 0.0, // A=1
        ];
        let mut f = DiscreteFactor::new(scope, logp).unwrap();
        f.normalize_rows(VarId(1)).unwrap();

        // check each A row sums to 1
        for a in 0..2usize {
            let mut s = 0.0f32;
            for c in 0..3usize {
                let lp = f.log_value_aligned(&[a, c]);
                s += lp.exp();
            }
            assert!(approx(s, 1.0, 1e-5), "sum={s}");
        }
    }

    #[test]
    fn marginalize_identity() {
        // f(A,B) = logp = idx; marginalize B -> g(A) should be lse over B.
        let scope = vec![VarSpec::new(0, 2), VarSpec::new(1, 3)];
        let mut logp = vec![0.0; 6];
        for i in 0..6 {
            logp[i] = (i as f32) * 0.1;
        }
        let f = DiscreteFactor::new(scope, logp).unwrap();

        let g = f.marginalize(&[VarId(1)]).unwrap(); // sum out B
        assert_eq!(g.dims(), &[2]);

        for a in 0..2usize {
            let mut row = Vec::new();
            for b in 0..3usize {
                row.push(f.log_value_aligned(&[a, b]));
            }
            let want = crate::logsumexp(&row);
            let got = g.log_value_aligned(&[a]);
            assert!(approx(got, want, 1e-6), "a={a} got={got} want={want}");
        }
    }
}
