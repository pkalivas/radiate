# --8<-- [start:elite_selector]
import radiate as rd

selector = rd.EliteSelector()
selector = rd.Select.elite()  # Using the Select dsl syntax
# --8<-- [end:elite_selector]

# --8<-- [start:tournament_selector]
import radiate as rd

selector = rd.TournamentSelector(k=3)
selector = rd.Select.tournament(k=3)  # Using the Select dsl syntax
# --8<-- [end:tournament_selector]

# --8<-- [start:roulette_selector]
import radiate as rd

selector = rd.RouletteSelector()
selector = rd.Select.roulette()  # Using the Select dsl syntax
# --8<-- [end:roulette_selector]

# --8<-- [start:boltzmann_selector]
import radiate as rd

selector = rd.BoltzmannSelector(temp=4.0)
selector = rd.Select.boltzmann(4.0)  # Using the Select dsl syntax
# --8<-- [end:boltzmann_selector]

# --8<-- [start:nsga2_selector]
import radiate as rd

selector = rd.NSGA2Selector()
selector = rd.Select.nsga2()  # Using the Select dsl syntax
# --8<-- [end:nsga2_selector]

# --8<-- [start:nsga3_selector]
import radiate as rd

selector = rd.NSGA3Selector(points=12)
selector = rd.Select.nsga3(12)  # Using the Select dsl syntax
# --8<-- [end:nsga3_selector]

# --8<-- [start:tournament_nsga2_selector]
import radiate as rd

selector = rd.TournamentNSGA2Selector()
selector = rd.Select.tournament_nsga2()  # Using the Select dsl syntax
# --8<-- [end:tournament_nsga2_selector]

# --8<-- [start:stochastic_sampling_selector]
import radiate as rd

selector = rd.StochasticSamplingSelector()
selector = rd.Select.stochastic_universal_sampling()  # Using the Select dsl syntax
# --8<-- [end:stochastic_sampling_selector]

# --8<-- [start:rank_selector]
import radiate as rd

selector = rd.RankSelector()
selector = rd.Select.rank()  # Using the Select dsl syntax
# --8<-- [end:rank_selector]

# --8<-- [start:linear_rank_selector]
import radiate as rd

selector = rd.LinearRankSelector(pressure=0.1)
selector = rd.Select.linear_rank(0.1)  # Using the Select dsl syntax
# --8<-- [end:linear_rank_selector]

_ = selector
