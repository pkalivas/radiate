
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
pub fn memory_derivative(current_output: &Vec<f32>, previous_output: &Vec<f32>, previous_memory: &Vec<f32>) -> Vec<f32> {
    assert!(current_output.len() == previous_output.len(), "Memory Derivative vector shapes do not match (curr out == prev out)");
    assert!(current_output.len() == previous_memory.len(), "Memory Derivative vector shapes do not match (curr out == prev mem)");
    current_output
        .iter()
        .zip(previous_output
            .iter()
            .zip(previous_memory.iter()))
        .map(|(o, (po, pm))| {
            o * (po + pm)
        })
        .collect::<Vec<_>>()
}


#[inline]
pub fn output_derivative(current_memory: &Vec<f32>, previous_output: &Vec<f32>) -> Vec<f32> {
    assert!(current_memory.len() == previous_output.len(), "Output derivative vector shapes don't match");
    current_memory
        .iter()
        .zip(previous_output.iter())
        .map(|(m, o)| m * o)
        .collect::<Vec<_>>()
}



#[inline]
pub fn input_derivative(current_state: &Vec<f32>, state_diff: &Vec<f32>) -> Vec<f32> {
    assert!(current_state.len() == state_diff.len(), "Input derivative vector shapes don't match");
    current_state
        .iter()
        .zip(state_diff.iter())
        .map(|(s, d)| s * d)
        .collect::<Vec<_>>()
}


#[inline]
pub fn state_derivative(current_input: &Vec<f32>, state_diff: &Vec<f32>) -> Vec<f32> {
    assert!(current_input.len() == state_diff.len(), "State derivative vector shapes don't match.");
    current_input
        .iter()
        .zip(state_diff.iter())
        .map(|(i, s)| i * s)
        .collect::<Vec<_>>()
}



#[inline]
pub fn forget_derivative(previous_memory: &Vec<f32>, state_diff: &Vec<f32>) -> Vec<f32> {
    assert!(previous_memory.len() == state_diff.len(), "Forget derivative vector shapes don't match");
    previous_memory
        .iter()
        .zip(state_diff.iter())
        .map(|(m, s)| m * s)
        .collect::<Vec<_>>()
}

