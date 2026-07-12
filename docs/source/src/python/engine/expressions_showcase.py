import radiate as rd

# Setup (not shown): a tiny regression dataset so the showcase runs standalone.
# This file is a long-running, plotting end-to-end demo — it is intentionally excluded
# from the fast snippet test suite (see py-radiate/tests/docs/test_user_guide.py), but its
# API is exercised by the focused expressions.py snippets and its include is build-checked.
inputs = [[x / 10.0] for x in range(-10, 11)]
answers = [[v[0] * 2.0] for v in inputs]

# --8<-- [start:example]
import radiate as rd

target_species = 4.0
rolling = int(target_species)

spec_count_signal = (
    rd.Expr.select("count.species").rolling(rolling).mean() / target_species
)
spec_dist_signal = (
    rd.Expr.select("species.distance").mean().rolling(rolling).mean() / target_species
)
spec_thresh_signal = rd.Expr.select("species.threshold").rolling(rolling).mean()
spec_evenness_signal = rd.Expr.select("species.evenness").rolling(rolling).mean()

distance_signal = (
    (rd.Expr.lit(0.9) * spec_count_signal)
    + (rd.Expr.lit(0.4) * spec_dist_signal)
    + (rd.Expr.lit(0.2) * spec_thresh_signal)
    + (rd.Expr.lit(0.1) * spec_evenness_signal)
).clamp(0.01, 10.0)

distance_signal_mean = distance_signal.mean()
species_count_mean = rd.Expr.select("count.species").mean().rolling(10).mean()

collector = rd.MetricCollector()

engine = (
    rd.Engine.graph(
        shape=(1, 1),
        vertex=[rd.Op.sub(), rd.Op.mul(), rd.Op.linear()],
        edge=rd.Op.weight(),
        output=rd.Op.linear(),
    )
    .regression(inputs, answers, loss=rd.MSE)
    .subscribe(collector)
    .diversity(rd.NeatDistance(), distance_signal)
    .metrics(
        distance_signal_mean=distance_signal_mean, species_count_mean=species_count_mean
    )
    .alters(
        rd.Cross.graph(0.05, 0.5),
        rd.Mutate.op(0.07, 0.05),
        rd.Mutate.graph(0.1, 0.1, False),
    )
    .limit(rd.Limit.score(0.001), rd.Limit.generations(1000))
)

result = engine.run(log=True)

collector.plot(
    "count.species",
    "distance_signal_mean",
    "species_count_mean",
)
# --8<-- [end:example]
