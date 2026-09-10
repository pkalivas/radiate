import radiate as rd

# TARGET_NUM = 30


# engine = rd.Engine.bit(TARGET_NUM).fitness(sum).limit(rd.Limit.score(TARGET_NUM))

# print(engine.run())


def fit(x: list[list[bool]]) -> int:
    sum_one = sum(1 for bit in x[0] if bit)
    sum_two = sum(1 for bit in x[1] if not bit)
    return sum_one + sum_two


engine = (
    rd.Engine.bit([20, 20])
    .fitness(fit)
    .minimizing()
    # .alter(rd.Mutate.bit_flip(0.01), rd.Cross.uniform())
    .limit(rd.Limit.score(0), rd.Limit.generations(200))
)

result = engine.run()

print(result.metrics().dashboard())
