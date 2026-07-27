# --8<-- [start:hamming]
import radiate as rd

diversity = rd.Dist.hamming()  # using the dsl syntax
# --8<-- [end:hamming]

# --8<-- [start:euclidean]
import radiate as rd

diversity = rd.Dist.euclidean()  # using the dsl syntax
# --8<-- [end:euclidean]

# --8<-- [start:cosine]
import radiate as rd

diversity = rd.Dist.cosine()  # using the dsl syntax
# --8<-- [end:cosine]

# --8<-- [start:neat]
import radiate as rd

# Parameters are: c1, c2, c3 - coefficients for excess genes, disjoint genes,
# and average weight differences respectively
diversity = rd.Dist.neat(0.1, 1.0, 0.5)  # using the dsl syntax
# --8<-- [end:neat]
