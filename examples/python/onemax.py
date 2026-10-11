import numpy as np
import radiate as rd

rd.random.seed(514)

NUM_BITS = 10000


# def fit(val: np.ndarray) -> int:
#     return np.bitwise_count(val).sum()


@rd.fitness(batch=True)
def fit(val: np.ndarray) -> np.ndarray:
    return np.bitwise_count(val).sum(axis=1)


engine = (
    rd.Engine.packed_bit(NUM_BITS, use_numpy=False, words=True)
    .fitness(fit)
    .alter(rd.Cross.multipoint(0.7, 2), rd.Mutate.bit_flip(0.2 / NUM_BITS))
    .limit(rd.Limit.score(NUM_BITS))
)

print(engine.run().duration())


# durations = []

# for i in range(50):
#     engine = (
#         rd.Engine.packed_bit(NUM_BITS, use_numpy=True, words=True)
#         .fitness(fit)
#         .alter(rd.Cross.multipoint(0.7, 2), rd.Mutate.bit_flip(0.2 / NUM_BITS))
#         .limit(rd.Limit.score(NUM_BITS))
#     )

#     print(f"Run {i + 1}")
#     duration = engine.run().duration().microseconds
#     print(f"Duration: {duration}")
#     durations.append(duration)

# print(durations)
# total_duration = sum(durations)
# total_time = timedelta(microseconds=total_duration)
# mean_duration = total_duration / len(durations)
# mean_time = timedelta(microseconds=mean_duration)

# print(f"Total duration: {total_duration}")
# print(f"Total time: {total_time}")
# print(f"Mean duration: {mean_duration}")
# print(f"Mean time: {mean_time}")

# # Total duration: 18868815
# # Total time: 0:00:18.868815
# # Mean duration: 377376.3
# # Mean time: 0:00:00.377376
