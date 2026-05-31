
# Example

Let's look at a basic example of how to use the `Codec` for evolving a simple function: finding the best values for `y = ax + b` where we want to find optimal values for `a` and `b`.

=== ":fontawesome-brands-python: Python"

    Python also allows you to pass a flag to most codecs to specify if you want a `numpy.array` or a `list` to be returned when decoding. You can do this by passing `use_numpy=True` to the codec constructor. 

    E.g. `rd.FloatCodec(shape=2, init_range=(-1.0, 1.0), bounds=(-10.0, 10.0), use_numpy=True)` will return a `numpy.array` when decoding. You can also just write the decoded value in your `fitness_func` in a `numpy.array(my_decoded_value)` format to get a `numpy.array` back. The performance difference between the two is negligible, so you can choose the one that best fits your needs.

    ---

    ## Building a python engine

    We just went over the `codec` and how to use it, now let's see how it translates into an actual engine. In python, `radiate`'s engine is built using a builder pattern, which allows you to chain method calls together to setup your engine. To make this easier, the `Engine` class itself has helper functions that actually construct the `codec` for you - same parameters, just wrapped, more fluid, and with better type hinting. Below we'll use both methods, but moving forward we'll use the helper function.

    The things below that look unfamiliar (`rd.Limit...`) will be covered in later sections, but for now just know that they are stopping conditions for the engine. The engine will stop when either of those conditions are met.

    ```python
    --8<-- "python/genome/example.py:example"
    ```

=== ":fontawesome-brands-rust: Rust"

    ```rust
    --8<-- "rust/genome/example.rs:codec_example"
    ```

    Some chromosomes are able to be used directly as codecs aswell. This means that you could create 
    engines using methods as seen below. For the duration of the user guide however, we'll use the above method.

    ```rust
    --8<-- "rust/genome/example.rs:chromosome_codec_example"    
    ```
