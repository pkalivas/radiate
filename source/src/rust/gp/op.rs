use radiate::prelude::*;

fn main() {
    // --8<-- [start:ops]
    // Example usage of an Op
    let fn_op = Op::add();
    let result = fn_op.eval(&[1.0, 2.0]); // result is 3.0

    // Example usage of a constant Op
    let const_op = Op::constant(42.0);
    let result = const_op.eval(&[]); // result is 42.0

    // Example usage of a variable Op
    let var_op = Op::var(0); // Read from input at index 0
    let inputs = var_op.eval(&[5.0, 10.0]); // result is 5.0 when evaluated with inputs
    // --8<-- [end:ops]

    // --8<-- [start:custom_op]
    fn my_square_op(inputs: &[f32]) -> f32 {
        inputs[0] * inputs[0]
    }

    // Supply a name, arity (number of inputs), and function to create the Op.
    // Square takes a single input, so its arity is Exact(1).
    let square_op = Op::Fn("Square", Arity::Exact(1), my_square_op);
    // --8<-- [end:custom_op]

    // --8<-- [start:operation_mutator]
    // Create a mutator that has a 10% chance to mutate an op and a 50% chance to replace it with a new one
    let mutator = OperationMutator::new(0.1, 0.5);
    // --8<-- [end:operation_mutator]
}
