use num_traits::ToPrimitive;
use radiate_utils::Primitive;

pub struct DistShape {
    /// Number of distinct values (richness).
    pub unique: usize,
    /// Pielou evenness of the value frequencies in `[0, 1]` — how uniformly the
    /// population is spread across distinct fitness values. Low evenness means
    /// the population has piled onto a few scores (premature convergence).
    pub evenness: f32,
    /// Gini coefficient of the distribution in `[0, 1]` — fitness inequality /
    /// selection pressure. 0 = every member shares the same score, → 1 = one
    /// member dominates.
    pub gini: f32,
}

/// Single descending-cost pass over an **ascending-sorted** slice computing its
/// richness, Pielou evenness, and Gini coefficient together. `sorted` must be
/// sorted ascending; callers already sort for the unique-count walk.
/// O(m) time, O(1) extra space.
///
/// The input values are generic over [`Primitive`], but the running statistics
/// are accumulated in `f32` (each value is converted once via [`Primitive::extract`]).
#[inline]
pub fn shape<P: Primitive>(sorted: &[P]) -> DistShape {
    let m = sorted.len();
    if m == 0 {
        return DistShape {
            unique: 0,
            evenness: 0.0,
            gini: 0.0,
        };
    }

    // Gini is only defined for non-negative quantities; if the distribution
    // dips below zero (e.g. minimization or signed fitness), shift it up so its
    // floor sits at zero. This keeps the standard formula well-defined across
    // arbitrary fitness scales at the cost of making it translation-invariant.
    // `safe_sub` rather than `-sorted[0]` because `Primitive` (which includes
    // unsigned ints) carries no `Neg` bound; the branch is unreachable for them.
    let shift = if sorted[0] < P::ZERO {
        P::ZERO.safe_sub(sorted[0])
    } else {
        P::ZERO
    };

    let total = m as f32;
    let mut unique = 0_f32;
    let mut entropy = 0_f32; // Shannon entropy of the value frequencies
    let mut run_len = 0_f32; // length of the current run of equal values
    let mut last: Option<P> = None;

    let mut gini_sum = 0.0_f32; // Σ (xᵢ + shift)
    let mut gini_weighted = 0.0_f32; // Σ (i+1)(xᵢ + shift), 1-indexed

    for (i, &val) in sorted.iter().enumerate() {
        match last {
            Some(l) if l.is_equal(val) => run_len += 1.0,
            _ => {
                if run_len > 0.0 {
                    let p = run_len / total;
                    entropy -= p * p.ln();
                }

                unique += 1.0;
                run_len = 1.0;
                last = Some(val);
            }
        }

        let x = val.safe_add(shift).extract::<f32>().unwrap_or(0.0);
        gini_sum += x;
        gini_weighted += (i as f32 + 1.0) * x;
    }

    if run_len > 0.0 {
        let p = run_len / total;
        entropy -= p * p.ln();
    }

    let evenness = if unique > 1.0 {
        // Bounded by 1.0 in theory; clamp away f32 rounding overshoot.
        (entropy / unique.ln()).min(1.0)
    } else {
        0.0
    };

    let gini = if gini_sum > 0.0 {
        ((2.0 * gini_weighted) / (total * gini_sum) - (total + 1.0) / total).max(0.0)
    } else {
        0.0
    };

    DistShape {
        unique: unique as usize,
        evenness,
        gini,
    }
}

/// Pielou evenness of a categorical distribution given each category's weight
/// (e.g. species member counts, histogram bin counts, fitness shares), in
/// `[0, 1]`.
///
/// This is the same measure as [`DistShape::evenness`], but for data whose
/// per-category weights are already known — rather than a flat sample whose
/// frequencies [`shape`] has to derive by counting runs of equal values:
/// - `~1.0`: every non-empty category holds the same weight — maximally even.
/// - `→ 0`: the weight is concentrated in a few categories.
///
/// Generic over any numeric weight (`usize` counts, integer or float shares);
/// non-positive entries are treated as empty. Normalized by `ln(k)` where `k`
/// is the number of non-empty categories; returns `0.0` when fewer than two
/// categories carry any weight (no diversity to be even about). O(n) time,
/// O(1) extra space.
#[inline]
pub fn evenness<T: ToPrimitive>(weights: &[T]) -> f32 {
    let mut total = 0.0_f32;
    let mut weighted_log = 0.0_f32; // Σ wᵢ ln wᵢ
    let mut nonzero = 0_f32; // categories carrying weight

    for w in weights {
        let w = w.to_f32().unwrap_or(0.0);
        if w > 0.0 {
            total += w;
            weighted_log += w * w.ln();
            nonzero += 1.0;
        }
    }

    if nonzero <= 1.0 {
        return 0.0;
    }

    // Single-pass identity: H = -Σ pᵢ ln pᵢ with pᵢ = wᵢ/total expands to
    // ln(total) - (Σ wᵢ ln wᵢ)/total, so we never materialize the pᵢ.
    let entropy = total.ln() - weighted_log / total;
    (entropy / nonzero.ln()).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected {b}, got {a}");
    }

    #[test]
    fn evenness_empty_or_single_category_is_zero() {
        approx(evenness::<usize>(&[]), 0.0);
        approx(evenness(&[7]), 0.0);
        // A single non-empty category among empties still has nothing to spread.
        approx(evenness(&[0, 5, 0]), 0.0);
    }

    #[test]
    fn evenness_equal_counts_is_maximal() {
        approx(evenness(&[4, 4, 4]), 1.0);
        approx(evenness(&[1, 1]), 1.0);
    }

    #[test]
    fn evenness_accepts_fractional_weights() {
        // Same shape as [15,10,5] counts, expressed as float shares → same J.
        approx(evenness(&[1.5_f32, 1.0, 0.5]), 0.9206);
    }

    #[test]
    fn evenness_skewed_counts_drops_below_one() {
        // [15,10,5]: H = 1.0115, H_max = ln 3 → ~0.921.
        approx(evenness(&[15, 10, 5]), 0.9206);
    }

    #[test]
    fn evenness_ignores_empty_categories_in_the_denominator() {
        // Empty categories carry no weight and are not counted toward k, so this
        // matches the two-category even case rather than being dragged down.
        approx(evenness(&[5, 0, 5, 0]), 1.0);
    }
}
