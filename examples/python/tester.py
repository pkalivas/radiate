import numpy as np
import radiate as rd

rd.random.seed(42)

# packed = rd.PackedBitCodec(65, use_numpy=False)

# encoded = packed.encode()

# print(encoded)

# decoded = packed.decode(encoded)
# print(type(decoded))
# print(decoded)

TARGET_NUM = 1000


def fit(val: np.ndarray) -> int:
    return np.bitwise_count(val).sum()


engine = (
    rd.Engine.packed_bit(TARGET_NUM, use_numpy=True, words=True)
    .fitness(fit)
    .alter(rd.Cross.multipoint(0.7, 2), rd.Mutate.bit_flip(0.02 / TARGET_NUM))
    .limit(rd.Limit.score(TARGET_NUM))
)

print(engine.run())

print(rd.Chromosome.float(length=1, init_range=(0.0, 1.0)))
print(rd.Chromosome.int(length=1, init_range=(0, 10)))
print(rd.Chromosome.bit(length=1))

print(rd.Chromosome.char(length=1, char_set=set("abcdefghijklmnopqrstuvwxyz")))


# def fit(x: list[list[bool]]) -> int:
#     sum_one = sum(1 for bit in x[0] if bit)
#     sum_two = sum(1 for bit in x[1] if not bit)
#     return sum_one + sum_two


# engine = (
#     rd.Engine.bit([20, 20])
#     .fitness(fit)
#     .minimizing()
#     .alter(rd.Mutate.bit_flip(0.01), rd.Cross.uniform())
#     .limit(rd.Limit.score(0), rd.Limit.generations(200))
# )

# result = engine.run()

# print(result)
