# --8<-- [start:uniform_mutator]
import radiate as rd

mutator = rd.UniformMutator(rate=0.1)
mutator = rd.UniformMutator(rate=rd.Rate.fixed(0.1))
mutator = rd.Mutate.uniform(rate=0.1)  # Using the Mutate dsl syntax
# --8<-- [end:uniform_mutator]

# --8<-- [start:gaussian_mutator]
import radiate as rd

mutator = rd.GaussianMutator(rate=0.1)
mutator = rd.Mutate.gaussian(rate=0.1)  # Using the Mutate dsl syntax
# --8<-- [end:gaussian_mutator]

# --8<-- [start:arithmetic_mutator]
import radiate as rd

mutator = rd.ArithmeticMutator(rate=0.1)
mutator = rd.Mutate.arithmetic(rate=0.1)  # Using the Mutate dsl syntax
# --8<-- [end:arithmetic_mutator]

# --8<-- [start:swap_mutator]
import radiate as rd

mutator = rd.SwapMutator(rate=0.1)
mutator = rd.Mutate.swap(rate=0.1)  # Using the Mutate dsl syntax
# --8<-- [end:swap_mutator]

# --8<-- [start:scramble_mutator]
import radiate as rd

mutator = rd.ScrambleMutator(rate=0.1)
mutator = rd.Mutate.scramble(rate=0.1)  # Using the Mutate dsl syntax
# --8<-- [end:scramble_mutator]

# --8<-- [start:invert_mutator]
import radiate as rd

mutator = rd.InversionMutator(rate=0.1)
mutator = rd.Mutate.inversion(rate=0.1)  # Using the Mutate dsl syntax
# --8<-- [end:invert_mutator]

# --8<-- [start:polynomial_mutator]
import radiate as rd

mutator = rd.PolynomialMutator(rate=0.1, eta=20.0)
mutator = rd.Mutate.polynomial(rate=0.1, eta=20.0)  # Using the Mutate dsl syntax
# --8<-- [end:polynomial_mutator]

# --8<-- [start:jitter_mutator]
import radiate as rd

mutator = rd.JitterMutator(rate=0.1, magnitude=0.5)
mutator = rd.Mutate.jitter(rate=0.1, magnitude=0.5)  # Using the Mutate dsl syntax
# --8<-- [end:jitter_mutator]
