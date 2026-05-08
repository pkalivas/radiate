use radiate_utils::{Float, MinMax};

#[inline]
pub fn scale_l1_affine<T: Float>(weights: &mut [T]) {
    let (min, max) = weights.iter().collect::<MinMax<T>>().min_max();
    affine_l1_internal(weights, min, max);
}

#[inline]
pub fn scale_l1_affine_sorted<T: Float>(weights: &mut [T]) {
    let first = *weights.first().unwrap_or(&T::ZERO);
    let last = *weights.last().unwrap_or(&T::ZERO);

    let (min, max) = if first < last {
        (first, last)
    } else {
        (last, first)
    };

    affine_l1_internal(weights, min, max);
}

#[inline]
pub fn scale_l1<T: Float>(weights: &mut [T]) {
    debug_assert!(
        weights.iter().all(|&x| x >= T::ZERO),
        "L1 normalization requires non-negative values"
    );

    let total = weights.iter().fold(T::ZERO, |acc, &x| acc + x);

    if total <= T::ZERO {
        return;
    }

    for score in weights.iter_mut() {
        *score = *score / total;
    }
}

#[inline]
fn affine_l1_internal<T: Float>(weights: &mut [T], min: T, max: T) {
    // Shift only if negatives exist
    let offset = if min < T::ZERO { -min } else { T::ZERO };

    let mut total = T::ZERO;
    for score in weights.iter_mut() {
        *score = (*score + offset) + T::EPS;
        total = total + *score;
    }

    if total <= T::ZERO || (max - min).abs() < T::EPS {
        let len_as_t = T::from(weights.len()).unwrap_or(T::ONE);
        for score in weights.iter_mut() {
            *score = T::ONE / len_as_t;
        }

        return;
    }

    for score in weights.iter_mut() {
        *score = *score / total;
    }
}
