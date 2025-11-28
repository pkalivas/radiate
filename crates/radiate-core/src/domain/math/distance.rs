#[inline]
pub fn euclidean(one: &[f32], two: &[f32]) -> f32 {
    one.iter()
        .zip(two.iter())
        .map(|(&a, &b)| {
            let diff = a - b;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
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
pub fn cosine(one: &[f32], two: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_one = 0.0;
    let mut norm_two = 0.0;

    for (&val_one, &val_two) in one.iter().zip(two.iter()) {
        dot_product += val_one * val_two;
        norm_one += val_one * val_one;
        norm_two += val_two * val_two;
    }

    if norm_one == 0.0 || norm_two == 0.0 {
        return 1.0;
    }

    1.0 - (dot_product / (norm_one.sqrt() * norm_two.sqrt()))
}
