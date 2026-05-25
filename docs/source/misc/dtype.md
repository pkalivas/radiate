
# Data Types

You may have noticed throughout these docs that we have snuck in a `dtype` argument here and there in the Python API. Radiate supports running certain `genes` & `chromosomes` with different data types in the backend (rust side), and this `dtype` argument allows you to specify which data type you want to use. This can seem like a minor detail, but in reality this can have a significant impact on the performance of your engine.

---

Radiate supports the following data types:

## Integers

- `int8`: 8-bit signed integer
- `int16`: 16-bit signed integer
- `int32`: 32-bit signed integer
- `int64`: 64-bit signed integer - this is the default data type for integers in radiate's python API
- `uint8`: 8-bit unsigned integer
- `uint16`: 16-bit unsigned integer
- `uint32`: 32-bit unsigned integer
- `uint64`: 64-bit unsigned integer

## Floats

- `float32`: 32-bit floating point number 
- `float64`: 64-bit floating point number - this is the default data type for floating point numbers in radiate's python API

## Why

The choice of data type can have a significant impact on the performance of your engine. For example, using `int8` instead of `int64` can reduce the memory footprint of your engine by a factor of 8, which can lead to faster execution times. However, if for example, you are optimizing an int engine (`rd.Engine.int(..., dtype=rd.Int8)`) and are decoding to a normal Python int, the fitness function will receive a Python int, which is typically a 64-bit integer. If you are using `int8` in the backend, this means that the values will be upcasted to `int64` when they are decoded. On the flip-side, if you are decoding to a numpy array, the fitness function will receive a numpy array of the same data type as the backend, which can lead to faster execution times.

## Example

Let's take a quick look at an example of where specifying the data type can lead to significant performance improvements. In this example, we will be solving the N-Queens problem using radiate. The N-Queens problem is a classic problem in which the goal is to place N queens on an N x N chessboard such that no two queens threaten each other. This means that no two queens can be in the same row, column, or diagonal.

Now, we could leave the `dtype` argument blank and let the engine optimize using `Int64` values in the backend, but since we know that the values will always be between 0 and N-1 (where N is the number of queens), we can use `UInt8`. Since we are also decoding to a numpy array, our fitness function will receive a numpy array of `uint8`s.

```python
--8<-- "python/misc/dtype_showcase.py:nqueens"
```

As a side note, we're also compiling this fitness function with [numba](https://numba.pydata.org/), which is a just-in-time compiler for Python. As a general statement, this example should run as fast (or almost as fast) as a pure rust implementation.