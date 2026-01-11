# Ops

The `ops` module provides sets of operations and formats for building and evolve genetic programs including `graphs` and `trees`. In the language of radiate, when using an `op`, it is the `Allele` of the `GraphNode` or `TreeNode`.
An `op` is a function that takes a number of inputs and returns a single output. The `op` can be a constant value, a variable, or a function that operates on the inputs.   

The `op` comes in five flavors:

1. **Function Operations**: Stateless functions that take inputs and return a value.
2. **Variable Operations**: Read from an input index, returning the value at that index.
3. **Constant Operations**: Fixed values that do not change - returning the value when called.
4. **Mutable Constant Operations**: Constants that can change over time, allowing for learnable parameters.
5. **Value Operations**: Stateful operations that maintain internal state and can take inputs to produce a value.

Each `op` has an `arity`, definingt the number of inputs it accepts. For example, the `Add` operation has an `arity` of 2 because it takes two inputs and returns their sum. The `Const` operation has an arity of 0 because it does not take any inputs, it just returns it's value. The `Var` operation has an arity of 0 because it takes an index as a parameter, and returns the value of the input at that index. 

Provided `Ops` include:

??? info "Basic ops"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `const` | 0 | x | `Op::constant()` | Const |
    | `named_const` | 0 | x | `Op::named_constant(name)` | Const |
    | `var` | 0 | Variable. input[i] - return the value of the input at index `i` | `Op::var(i)` | Var |
    | `identity` |1| return the input value | `Op::identity()` | Fn |


??? info "Basic math operations"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `Add` | 2 | x + y | `Op::add()` | Fn |
    | `Sub` | 2 | x - y | `Op::sub()` | Fn |
    | `Mul` | 2 | x * y | `Op::mul()` | Fn |
    | `Div` | 2 | x / y | `Op::div()` | Fn |
    | `Sum` | Any | Sum of n values | `Op::sum()` | Fn |
    | `Product` | Any | Product of n values | `Op::prod()` | Fn |
    | `Difference` | Any | Difference of n values | `Op::diff()` | Fn |
    | `Neg` | 1 | -x | `Op::neg()` | Fn |
    | `Abs` | 1 | abs(x) | `Op::abs()` | Fn |
    | `pow` | 2 | x^y | `Op::pow()` | Fn |
    | `Sqrt` | 1 | sqrt(x) | `Op::sqrt()` | Fn |
    | `Abs` | 1 | abs(x) | `Op::abs()` | Fn |
    | `Exp` | 1 | e^x | `Op::exp()` | Fn |
    | `Log` | 1 | log(x) | `Op::log()` | Fn |
    | `Sin` | 1 | sin(x) | `Op::sin()` | Fn |
    | `Cos` | 1 | cos(x) | `Op::cos()` | Fn |
    | `Tan` | 1 | tan(x) | `Op::tan()` | Fn |
    | `Max` | Any | Max of n values | `Op::max()` | Fn |
    | `Min` | Any | Min of n values | `Op::min()` | Fn |
    | `Ceil` | 1 | ceil(x) | `Op::ceil()` | Fn |
    | `Floor` | 1 | floor(x) | `Op::floor()` | Fn |
    | `Weight` | 1 | Weighted sum of n values | `Op::weight()` | MutableConst |

??? info "Activation Ops"

    These are the most common activation functions used in Neural Networks.

    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `Sigmoid` | Any | 1 / (1 + e^-x) | `Op::sigmoid()` | Fn |
    | `Tanh` | Any | tanh(x) | `Op::tanh()` | Fn |
    | `ReLU` | Any | max(0, x) | `Op::relu()` | Fn |
    | `LeakyReLU` | Any | x if x > 0 else 0.01x | `Op::leaky_relu()` | Fn |
    | `ELU` | Any | x if x > 0 else a(e^x - 1) | `Op::elu()` | Fn |
    | `Linear` | Any | Linear combination of n values | `Op::linear()` | Fn |
    | `Softplus` | Any | log(1 + e^x) | `Op::softplus()` | Fn |
    | `SELU` | Any | x if x > 0 else a(e^x - 1) | `Op::selu()` | Fn |
    | `Swish` | Any | x / (1 + e^-x) | `Op::swish()` | Fn |
    | `Mish` | Any | x * tanh(ln(1 + e^x)) | `Op::mish()` | Fn |
    
??? info "bool Ops"
    | Name | Arity | Description | Initalize | Type |
    |------|-------|-------------|----------|---- |
    | `And` | 2 | x && y | `Op::and()` | Fn |
    | `Or` | 2 | `x || y` | `Op::or()` | Fn |
    | `Not` | 1 | !x | `Op::not()` | Fn |
    | `Xor` | 2 | x ^ y | `Op::xor()` | Fn |
    | `Nand` | 2 | !(x && y) | `Op::nand()` | Fn |
    | `Nor` | 2 | `!(x || y)` | `Op::nor()` | Fn |
    | `Xnor` | 2 | !(x ^ y) | `Op::xnor()` | Fn |
    | `Equal` | 2 | x == y | `Op::eq()` | Fn |
    | `NotEqual` | 2 | x != y | `Op::ne()` | Fn |
    | `Greater` | 2 | x > y | `Op::gt()` | Fn |
    | `Less` | 2 | x < y | `Op::lt()` | Fn |
    | `GreaterEqual` | 2 | x >= y | `Op::ge()` | Fn |
    | `LessEqual` | 2 | x <= y | `Op::le()` | Fn |
    | `IfElse` | 3 | if x then y else z | `Op::if_else()` | Fn |

=== ":fontawesome-brands-python: Python"

    Ops in python can't be directly evaluated like in rust. However, they can still be constructed and used in a similar way.

    ```python
    import radiate as rd

    add = rd.Op.add()  
    sub = rd.Op.sub()
    mul = rd.Op.mul()
    div = rd.Op.div()

    constant = rd.Op.constant(42.0)
    variable = rd.Op.var(0)

    sigmoid = rd.Op.sigmoid()
    relu = rd.Op.relu()
    tanh = rd.Op.tanh()
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Example usage of an Op
    let fn_op = Op::add();
    let result = fn_op.eval(&[1.0, 2.0]); // result is 3.0

    // Example usage of a constant Op
    let const_op = Op::constant(42.0);
    let result = const_op.eval(&[]); // result is 42.0

    // Example usage of a variable Op
    let var_op = Op::var(0); // Read from input at index 0
    let inputs = var_op.eval(&[5.0, 10.0]); // result is 5.0 when evaluated with inputs
    ```

    Want to create your own `Op<T>`? Its pretty simple! Let create a custom `Square` operation that squares it's input.

    ```rust
    use std::sync::Arc;
    use radiate::*;

    fn my_square_op(inputs: &[f32]) -> f32 {
        inputs[0] * inputs[0]
    }

    // Supply a name, arity (number of inputs - 1 in this case), and function to create the Op
    let square_op = Op::new("Square", Arity::Exact(1), my_square_op);
    ```

    Now you have a new `square_op` which is completely compatible with the rest of the Radiate GP system and can be plugged in anywhere a regular `Op` can be used! For more information on creating ops, checkout the [API docs](https://docs.rs/radiate-gp/1.2.20/radiate_gp/ops/operation/enum.Op.html) to see how the rest are created - its not too crazy. 

### Alters

#### OperationMutator

> Inputs
> 
>   * `rate`: f32 - Mutation rate (0.0 to 1.0)
>   * `replace_rate`: f32 - Rate at which to replace an old `op` with a completely new one (0.0 to 1.0)

- **Purpose**: Randomly mutate an operation within a `TreeNode` or `GraphNode`.

This mutator randomly changes or alters the `op` of a node within a `TreeChromosome` or `GraphChromosome`. It can replace the `op` with a new one from the [store](node.md#store) or modify its parameters.

=== ":fontawesome-brands-python: Python"

    ```python
    import radiate as rd

    # Create a mutator that has a 10% chance to mutate an op and a 50% chance to replace it with a new one
    mutator = rd.OperationMutator(0.1, 0.5)
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    use radiate::*;

    // Create a mutator that has a 10% chance to mutate an op and a 50% chance to replace it with a new one
    let mutator = OperationMutator::new(0.1, 0.5);
    ```
