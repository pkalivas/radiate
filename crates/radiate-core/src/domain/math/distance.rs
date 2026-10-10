use radiate_utils::Float;

#[inline]
pub fn euclidean<F: Float>(one: &[F], two: &[F]) -> F {
    let mut sum = F::zero();
    for (&a, &b) in one.iter().zip(two.iter()) {
        let diff = a - b;
        sum = sum + diff * diff;
    }
    sum.sqrt()
}

#[inline]
pub fn hamming<T>(one: &[T], two: &[T]) -> f32
where
    T: PartialEq,
{
    one.iter()
        .zip(two.iter())
        .map(|(a, b)| if a != b { 1.0 } else { 0.0 })
        .sum::<f32>()
        / one.len() as f32
}

#[inline]
pub fn cosine<F: Float>(one: &[F], two: &[F]) -> F {
    let mut dot_product = F::zero();
    let mut norm_one = F::zero();
    let mut norm_two = F::zero();

    for (&val_one, &val_two) in one.iter().zip(two.iter()) {
        dot_product = dot_product + val_one * val_two;
        norm_one = norm_one + val_one * val_one;
        norm_two = norm_two + val_two * val_two;
    }

    if norm_one == F::zero() || norm_two == F::zero() {
        return F::one();
    }

    F::one() - (dot_product / (norm_one.sqrt() * norm_two.sqrt()))
}
