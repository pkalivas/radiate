# --8<-- [start:intro]
# --8<-- [start:fixed_rate]
# --8<-- [start:linear_rate]
# --8<-- [start:stepwise_rate]
# --8<-- [start:sine_rate]
# --8<-- [start:triangular_rate]
# --8<-- [start:exp_rate]


# rate = rd.Rate.fixed(0.1)

# rates = []
# for i in range(100):
#     rates.append(rate.value(i))
# --8<-- [end:intro]


# rate = rd.Rate.fixed(0.1)
# --8<-- [end:fixed_rate]


# rate = rd.Rate.linear(start=0.1, end=0.9, duration=25)
# --8<-- [end:linear_rate]


# steps = [(0, 0.1), (25, 0.5), (75, 0.9)]
# rate = rd.Rate.stepwise(steps)
# --8<-- [end:stepwise_rate]


# rate = rd.Rate.sine(min=0.1, max=0.9, periods=10)
# --8<-- [end:sine_rate]


# rate = rd.Rate.triangular(min=0.1, max=0.9, periods=10)
# --8<-- [end:triangular_rate]


# rate = rd.Rate.exp(start=0.5, end=0.1, half_life=25)
# --8<-- [end:exp_rate]
