
use super::super::activation::Activation;



/// multiply two vectors element wise
#[inline]
pub fn element_multiply(one: &mut Vec<f32>, two: &Vec<f32>) {
    assert!(one.len() == two.len(), "Element multiply vector shapes don't match");
    one.iter_mut()
        .zip(two.iter())
        .for_each(|(a, b)| {
            *a *= b
        });
}



/// invert a vector that is already holding values between 0 and 1
#[inline]
pub fn element_invert(one: &mut Vec<f32>) {
    one.iter_mut()
        .for_each(|a| *a = 1.0 - *a);
}



/// add elements from vectors together element wise
#[inline]
pub fn element_add(one: &mut Vec<f32>, two: &Vec<f32>) {
    assert!(one.len() == two.len(), "Element add vector shapes don't match");
    one.iter_mut()
        .zip(two.iter())
        .for_each(|(a, b)| {
            *a += b
        });
}


#[inline]
pub fn element_activate(one: &Vec<f32>, func: Activation) -> Vec<f32> {
    one.iter()
        .map(|x| {
            func.activate(*x)
        })
        .collect()
}


#[inline]
pub fn element_deactivate(one: &Vec<f32>, func: Activation) -> Vec<f32> {
    one.iter()
        .map(|x| {
            func.deactivate(*x)
        })
        .collect()
}


#[inline]
pub fn product(one: &Vec<f32>, two: &Vec<f32>) -> Vec<f32> {
    assert!(one.len() == two.len(), "Product dimensions do not match");
    one.iter()
        .zip(two.iter())
        .map(|(o, t)| o * t)
        .collect::<Vec<_>>()
}