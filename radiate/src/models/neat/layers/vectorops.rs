
use super::super::{
    activation::Activation,
    loss::Loss
};


/// multiply two vectors element-wise
#[inline]
pub fn element_multiply(one: &mut [f32], two: &[f32]) {
    assert!(one.len() == two.len(), "Element multiply vector shapes don't match");
    one.iter_mut()
        .zip(two.iter())
        .for_each(|(a, b)| {
            *a *= b
        });
}



/// invert a vector that is already holding values between 0 and 1
#[inline]
pub fn element_invert(one: &mut [f32]) {
    one.iter_mut()
        .for_each(|a| *a = 1.0 - *a);
}



/// add elements from vectors together element-wise
#[inline]
pub fn element_add(one: &mut [f32], two: &[f32]) {
    assert!(one.len() == two.len(), "Element add vector shapes don't match");
    one.iter_mut()
        .zip(two.iter())
        .for_each(|(a, b)| {
            *a += b
        });
}


#[inline]
pub fn element_activate(one: &[f32], func: Activation) -> Vec<f32> {
    one.iter()
        .map(|x| {
            func.activate(*x)
        })
        .collect()
}


#[inline]
pub fn element_deactivate(one: &[f32], func: Activation) -> Vec<f32> {
    one.iter()
        .map(|x| {
            func.deactivate(*x)
        })
        .collect()
}


#[inline]
pub fn product(one: &[f32], two: &[f32]) -> Vec<f32> {
    assert!(one.len() == two.len(), "Product dimensions do not match");
    one.iter()
        .zip(two.iter())
        .map(|(o, t)| o * t)
        .collect::<Vec<_>>()
}


#[inline]
pub fn subtract(one: &[f32], two: &[f32]) -> Vec<f32> {
    assert!(one.len() == two.len(), "Subtract lengths do not match");
    one.iter()
        .zip(two.iter())
        .map(|(tar, pre)| (tar - pre))
        .collect::<Vec<_>>()
}


#[inline]
pub fn softmax(one: &[f32]) -> Vec<f32> {
    let ex = one   
        .iter()
        .map(|x| x.exp())
        .collect::<Vec<_>>();
    let sum = ex.iter().sum::<f32>();
    ex.iter()
        .map(|x| x / sum)
        .collect()
}


#[inline]
pub fn d_softmax(one: &[f32]) -> Vec<f32> {
    one.iter()
        .map(|x| x - 1.0)
        .collect()
}



#[inline]
pub fn loss(one: &[f32], two: &[f32], loss_fn: &Loss) -> (f32, Vec<f32>) {
    assert!(one.len() == two.len(), "Loss vector shape don't match");
    match loss_fn {
        Loss::Diff => {
            let difference = subtract(one, two);
            let total = difference.iter().sum::<f32>();
            return (total, difference);
        },
        Loss::MSE => {
            let mut squared_error = 0.0;
            let errs = one.iter()
                .zip(two.iter())
                .map(|(i, j)| {
                    let e = (i - j).powf(2.0);
                    squared_error += e;
                    e
                })
                .collect::<Vec<_>>();
            return ((1.0 / one.len() as f32) * squared_error, errs);

        }
    }
}