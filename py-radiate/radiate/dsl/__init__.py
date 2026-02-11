# radiate/api.py (or radiate/dsl.py)

from radiate.operators.selector import (
    TournamentSelector,
    RouletteSelector,
    NSGA2Selector,
    NSGA3Selector,
    EliteSelector,
    BoltzmannSelector,
    RankSelector,
    LinearRankSelector,
    StochasticSamplingSelector,
    TournamentNSGA2Selector,
)
from radiate.operators.alterer import (
    BlendCrossover,
    IntermediateCrossover,
    ArithmeticMutator,
    SwapMutator,
    UniformCrossover,
    UniformMutator,
    MultiPointCrossover,
    MeanCrossover,
    ShuffleCrossover,
    SimulatedBinaryCrossover,
    PartiallyMappedCrossover,
    GaussianMutator,
    GraphCrossover,
    OperationMutator,
    GraphMutator,
    TreeCrossover,
    HoistMutator,
    InversionMutator,
    PolynomialMutator,
    EdgeRecombinationCrossover,
    JitterMutator,
    ScrambleMutator,
)
from radiate.operators.limit import (
    ScoreLimit,
    GenerationsLimit,
    SecondsLimit,
    ConvergenceLimit,
    MetricLimit,
)
from radiate.operators.distance import (
    EuclideanDistance,
    CosineDistance,
    NeatDistance,
    HammingDistance,
)

# from radiate.operators.rate import Rate


class Select:
    @staticmethod
    def tournament(k=3):
        return TournamentSelector(k=k)

    @staticmethod
    def roulette():
        return RouletteSelector()

    @staticmethod
    def nsga2():
        return NSGA2Selector()

    @staticmethod
    def nsga3(points=12):
        return NSGA3Selector(points=points)

    @staticmethod
    def elite():
        return EliteSelector()

    @staticmethod
    def boltzmann(temp=1.0):
        return BoltzmannSelector(temp=temp)

    @staticmethod
    def rank():
        return RankSelector()

    @staticmethod
    def linear_rank(pressure=1.5):
        return LinearRankSelector(pressure=pressure)

    @staticmethod
    def stochastic_universal_sampling():
        return StochasticSamplingSelector()

    @staticmethod
    def tournament_nsga2(k=3):
        return TournamentNSGA2Selector(k=k)


class Cross:
    @staticmethod
    def sbx(rate=0.1, contiguity=0.5):
        return SimulatedBinaryCrossover(rate, contiguity)

    @staticmethod
    def pmx(rate=0.1):
        return PartiallyMappedCrossover(rate)

    @staticmethod
    def multipoint(rate=0.1):
        return MultiPointCrossover(rate)

    @staticmethod
    def mean(rate=0.1):
        return MeanCrossover(rate)

    @staticmethod
    def uniform(rate=0.1):
        return UniformCrossover(rate)

    @staticmethod
    def blend(rate=0.1, alpha=0.5):
        return BlendCrossover(rate, alpha)

    @staticmethod
    def intermediate(rate=0.1, alpha=0.5):
        return IntermediateCrossover(rate, alpha)

    @staticmethod
    def shuffle(rate=0.1):
        return ShuffleCrossover(rate)

    @staticmethod
    def edge_recombination(rate=0.1):
        return EdgeRecombinationCrossover(rate)

    @staticmethod
    def graph(vertex_rate=0.1, edge_rate=0.1):
        return GraphCrossover(vertex_rate, edge_rate)

    @staticmethod
    def tree(rate=0.1):
        return TreeCrossover(rate)

    @staticmethod
    def operation(rate=0.1, replace_rate=0.1):
        return OperationMutator(rate, replace_rate)


class Mutate:
    @staticmethod
    def uniform(rate=0.1):
        return UniformMutator(rate)

    @staticmethod
    def gaussian(rate=0.1):
        return GaussianMutator(rate)

    @staticmethod
    def op(rate=0.1, replace_rate=0.1):
        return OperationMutator(rate, replace_rate)

    @staticmethod
    def graph(vertex_rate=0.1, edge_rate=0.1, allow_recurrent=True):
        return GraphMutator(vertex_rate, edge_rate, allow_recurrent)

    @staticmethod
    def scramble(rate=0.1):
        return ScrambleMutator(rate)

    @staticmethod
    def swap(rate=0.1):
        return SwapMutator(rate)

    @staticmethod
    def hoist(rate=0.1):
        return HoistMutator(rate)

    @staticmethod
    def inversion(rate=0.1):
        return InversionMutator(rate)

    @staticmethod
    def polynomial(rate=0.1, eta=20):
        return PolynomialMutator(rate, eta)

    @staticmethod
    def jitter(rate=0.1, magnitude=0.01):
        return JitterMutator(rate, magnitude)

    @staticmethod
    def arithmetic(rate=0.1):
        return ArithmeticMutator(rate)


class Limit:
    @staticmethod
    def score(value: float) -> ScoreLimit:
        return ScoreLimit(value)

    @staticmethod
    def generations(n: int) -> GenerationsLimit:
        return GenerationsLimit(n)

    @staticmethod
    def seconds(secs: int) -> SecondsLimit:
        return SecondsLimit(secs)

    @staticmethod
    def convergence(window: int, threshold: float) -> ConvergenceLimit:
        return ConvergenceLimit(window, threshold)

    @staticmethod
    def metric(
        name: str = "evaluation_count", limit=lambda metric: metric.sum() > 1000
    ):
        return MetricLimit(name, limit)


class Dist:
    @staticmethod
    def euclidean():
        return EuclideanDistance()

    @staticmethod
    def cosine():
        return CosineDistance()

    @staticmethod
    def neat(excess: float = 1.0, disjoint: float = 1.0, weight_diff: float = 3.0):
        return NeatDistance(excess, disjoint, weight_diff)

    @staticmethod
    def hamming():
        return HammingDistance()


__all__ = ["Select", "Mutate", "Cross", "Dist", "Limit"]
