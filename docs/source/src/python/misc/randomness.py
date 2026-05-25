# NOTE: not named random.py on purpose — that would shadow the stdlib `random` module.
# --8<-- [start:random]
import radiate as rd

# set a seed for reproducibility
rd.random.seed(42)

# random float
rand_float = rd.random.float(min=0.0, max=1.0)

# random integer
rand_int = rd.random.int(min=0, max=10)

# random bool with 50% chance of being true
rand_bool = rd.random.bool(prob=0.5)

# randomly sample 2 elements from a list
rand_choice = rd.random.sample(data=[1, 2, 3, 4, 5], count=2)

# choose a random element from a list
rand_element = rd.random.choose(data=[1, 2, 3, 4, 5])
# --8<-- [end:random]
