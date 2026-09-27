# Ops

The `ops` module provides sets of operations and formats for building and evolving genetic programs including `graphs` and `trees`. In the language of radiate, when using an `op`, it is the `Allele` of the `GraphNode` or `TreeNode`.
An `op` is a function that takes a number of inputs and returns a single output. The `op` can be a constant value, a variable, or a function that operates on the inputs.   

The `op` comes in five flavors, mirroring the variants of the `Op<T>` enum:

1. **Function (`Fn`)**: Stateless functions that take inputs and return a value (e.g. `Add`, `Sigmoid`).
2. **Variable (`Var`)**: Reads from an input vector, returning the value at that index.
3. **Constant (`Const`)**: A fixed value that does not change - returning the value when called.
4. **Value (`Value`)**: A stateful operation that holds data (a `Param<T>`) alongside a function, allowing for learnable parameters such as the `Weight` op.
5. **Pair (`Pair`)**: Like `Value`, but the stored data is a pair of values (a `Param<(T, T)>`), such as the two learnable weights of the `Weight2` op.

Each `op` has an `arity`, defining the number of inputs it accepts. For example, the `Add` operation has an `arity` of 2 because it takes two inputs and returns their sum. The `Const` operation has an arity of 0 because it does not take any inputs, it just returns its value. The `Var` operation has an arity of 0 because it takes an index as a parameter, and returns the value of the input at that index. 

Provided `Ops` include:

??? info "Basic ops"
    | Name | Arity | Description | Initialize | Type |
    |------|-------|-------------|----------|---- |
    | `const` | 0 | A fixed constant value | `Op::constant(value)` | Const |
    | `named_const` | 0 | A constant value with an associated name | `Op::named_constant(name, value)` | Const |
    | `var` | 0 | Variable. input[i] - return the value of the input at index `i` | `Op::var(i)` | Var |
    | `vars` | 0 | One `var` for each index in a range | `Op::vars(range)` | Var |
    | `named_var` | 0 | A `var` with a custom name instead of `Xi` | `Op::named_var(name, i)` | Var |
    | `category` | 0 | A `var` for a categorical input with `k` possible values | `Op::category(name, i, k)` | Var |
    | `identity` |1| return the input value | `Op::identity()` | Fn |

    !!! note "Python"
        Python provides `const` (as `Op.const(value)`), `var` (as `Op.var(i)`) and `identity`. `named_const`, `vars`, `named_var` and `category` are Rust-only.


??? info "Basic math operations"
    | Name | Arity | Description | Initialize | Type |
    |------|-------|-------------|----------|---- |
    | `Add` | 2 | x + y | `Op::add()` | Fn |
    | `Sub` | 2 | x - y | `Op::sub()` | Fn |
    | `Mul` | 2 | x * y | `Op::mul()` | Fn |
    | `Div` | 2 | x / y (1 if y ≈ 0) | `Op::div()` | Fn |
    | `Sum` | Any | Sum of n values | `Op::sum()` | Fn |
    | `Product` | Any | Product of n values | `Op::prod()` | Fn |
    | `Difference` | Any | Difference of n values | `Op::diff()` | Fn |
    | `Neg` | 1 | -x | `Op::neg()` | Fn |
    | `Abs` | 1 | abs(x) | `Op::abs()` | Fn |
    | `pow` | 2 | x^y | `Op::pow()` | Fn |
    | `Sqrt` | 1 | sqrt(x) | `Op::sqrt()` | Fn |
    | `Exp` | 1 | e^x | `Op::exp()` | Fn |
    | `Log` | 1 | ln(x) (0 if x ≤ 0) | `Op::log()` | Fn |
    | `Sin` | 1 | sin(x) | `Op::sin()` | Fn |
    | `Cos` | 1 | cos(x) | `Op::cos()` | Fn |
    | `Tan` | 1 | tan(x) | `Op::tan()` | Fn |
    | `Max` | Any | Max of n values | `Op::max()` | Fn |
    | `Min` | Any | Min of n values | `Op::min()` | Fn |
    | `Ceil` | 1 | ceil(x) | `Op::ceil()` | Fn |
    | `Floor` | 1 | floor(x) | `Op::floor()` | Fn |
    | `Sign` | 1 | -1, 0 or 1, matching the sign of x | `Op::sign()` | Fn |
    | `Reciprocal` | 1 | 1 / x (1 if x ≈ 0) | `Op::reciprocal()` | Fn |
    | `LogSumExp` | 2 | ln(e^x + e^y) | `Op::logsumexp()` | Fn |
    | `Gaussian` | Any | e^(-x²), where x is the sum of the inputs | `Op::gaussian()` | Fn |
    | `Tooth` | Any | x mod 1 (keeps the sign of x), where x is the sum of the inputs | `Op::tooth()` | Fn |
    | `Weight` | 1 | x * w (input multiplied by a learnable weight) | `Op::weight()` | Value |
    | `Weight2` | 2 | x * w1 + y * w2 (each input multiplied by its own learnable weight) | `Op::weight2()` | Pair |

    All of these clamp their output to ±1e10 and return 0 instead of NaN, so an evolved program can't overflow or produce invalid values. "≈ 0" means within 1e-7 of zero.

    Learnable weights start at a random value in [-1, 1]. To start from a specific value, use `Op::weight_with(w)` or `Op::weight2_with((w1, w2))`.

    !!! note "Python"
        `LogSumExp`, `Op::weight_with` and `Op::weight2_with` are Rust-only. Every other op in this table is available in Python.

??? info "Activation Ops"

    These are the most common activation functions used in Neural Networks. Each accepts any number of inputs, **sums them first**, then applies the activation — so `x` in the descriptions below refers to the sum of the node's inputs.

    | Name | Arity | Description | Initialize | Type |
    |------|-------|-------------|----------|---- |
    | `Sigmoid` | Any | 1 / (1 + e^-x) | `Op::sigmoid()` | Fn |
    | `Tanh` | Any | tanh(x) | `Op::tanh()` | Fn |
    | `ReLU` | Any | max(0, x) | `Op::relu()` | Fn |
    | `LeakyReLU` | Any | x if x > 0 else 0.5x | `Op::leaky_relu()` | Fn |
    | `ELU` | Any | x if x > 0 else 0.5(e^x - 1) | `Op::elu()` | Fn |
    | `Linear` | Any | x (sum of inputs, identity activation) | `Op::linear()` | Fn |
    | `Softplus` | Any | log(1 + e^x) | `Op::softplus()` | Fn |
    | `Swish` | Any | x / (1 + e^-x) | `Op::swish()` | Fn |
    | `Mish` | Any | x * tanh(ln(1 + e^x)) | `Op::mish()` | Fn |
    
??? info "bool Ops"
    | Name | Arity | Description | Initialize | Type |
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
    | `AndThen` | 3 | x && y && z | `Op::and_then()` | Fn |
    | `Implies` | 2 | `!x || y` | `Op::implies()` | Fn |
    | `Iff` | 2 | x == y | `Op::iff()` | Fn |

    !!! note "Python"
        The `bool` ops are Rust-only.

=== ":fontawesome-brands-python: Python"

    ```python
    --8<-- "python/gp/op.py:ops"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/gp/op.rs:ops"
    ```

    Want to create your own `Op<T>`? It's pretty simple! But beware, `radiate` cannot serialize/deserialize custom ops. Let's create a custom `Square` operation that squares its input.

    ```rust
    --8<-- "rust/gp/op.rs:custom_op"
    ```

    Now you have a new `square_op` which is completely compatible with the rest of the Radiate GP system and can be plugged in anywhere a regular `Op` can be used! For more information on creating ops, check out the [API docs](https://docs.rs/radiate-gp/latest/radiate_gp/ops/operation/enum.Op.html) to see how the rest are created - it's not too crazy. 

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
    --8<-- "python/gp/op.py:operation_mutator"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/gp/op.rs:operation_mutator"
    ```

