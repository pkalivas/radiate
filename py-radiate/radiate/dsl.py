# radiate/api.py (or radiate/dsl.py)

from .inputs.selector import (
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
from .inputs.alterer import (
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
from .inputs.limit import (
    ScoreLimit,
    GenerationsLimit,
    SecondsLimit,
    ConvergenceLimit,
    MetricLimit,
)
from .inputs.distance import (
    EuclideanDistance,
    CosineDistance,
    NeatDistance,
    HammingDistance,
)

from .inputs.rate import Rate


class Select:
    tournament = staticmethod(lambda k=3: TournamentSelector(k=k))
    roulette = staticmethod(lambda: RouletteSelector())
    nsga2 = staticmethod(lambda: NSGA2Selector())
    nsga3 = staticmethod(lambda points=12: NSGA3Selector(points=points))
    elite = staticmethod(lambda: EliteSelector())
    boltzmann = staticmethod(lambda temp=1.0: BoltzmannSelector(temp=temp))
    rank = staticmethod(lambda: RankSelector())
    linear_rank = staticmethod(
        lambda pressure=1.5: LinearRankSelector(pressure=pressure)
    )
    stochastic_universal_sampling = staticmethod(lambda: StochasticSamplingSelector())
    tournament_nsga2 = staticmethod(lambda k=3: TournamentNSGA2Selector(k=k))


class Cross:
    sbx = staticmethod(
        lambda rate=0.1, contiguity=0.5: SimulatedBinaryCrossover(rate, contiguity)
    )
    pmx = staticmethod(lambda rate=0.1: PartiallyMappedCrossover(rate))
    multi_point = staticmethod(lambda rate=0.1: MultiPointCrossover(rate))
    mean = staticmethod(lambda rate=0.1: MeanCrossover(rate))
    uniform = staticmethod(lambda rate=0.1: UniformCrossover(rate))
    blend = staticmethod(lambda rate=0.1, alpha=0.5: BlendCrossover(rate, alpha))
    intermediate = staticmethod(
        lambda rate=0.1, alpha=0.5: IntermediateCrossover(rate, alpha)
    )
    graph = staticmethod(
        lambda vertex_rate=0.1, edge_rate=0.1: GraphCrossover(vertex_rate, edge_rate)
    )
    operation = staticmethod(
        lambda rate=0.1, replace_rate=0.1: OperationMutator(rate, replace_rate)
    )
    tree = staticmethod(lambda rate=0.1: TreeCrossover(rate))
    edge_recombination = staticmethod(lambda rate=0.1: EdgeRecombinationCrossover(rate))
    shuffle = staticmethod(lambda rate=0.1: ShuffleCrossover(rate))


class Mutate:
    uniform = staticmethod(lambda rate=0.1: UniformMutator(rate))
    gaussian = staticmethod(lambda rate=0.1: GaussianMutator(rate))
    operation = staticmethod(
        lambda rate=0.1, replace_rate=0.1: OperationMutator(rate, replace_rate)
    )
    graph = staticmethod(
        lambda vertex_rate=0.1, edge_rate=0.1, allow_recurrent=True: GraphMutator(
            vertex_rate, edge_rate, allow_recurrent
        )
    )
    scramble = staticmethod(lambda rate=0.1: ScrambleMutator(rate))
    swap = staticmethod(lambda rate=0.1: SwapMutator(rate))
    hoist = staticmethod(lambda rate=0.1: HoistMutator(rate))
    inversion = staticmethod(lambda rate=0.1: InversionMutator(rate))
    polynomial = staticmethod(lambda rate=0.1, eta=20: PolynomialMutator(rate, eta))
    jitter = staticmethod(
        lambda rate=0.1, magnitude=0.01: JitterMutator(rate, magnitude)
    )
    arithmetic = staticmethod(lambda rate=0.1: ArithmeticMutator(rate))


class Limit:
    score = staticmethod(lambda value: ScoreLimit(value))
    generations = staticmethod(lambda n: GenerationsLimit(n))
    seconds = staticmethod(lambda secs: SecondsLimit(secs))
    convergence = staticmethod(
        lambda window, threshold: ConvergenceLimit(window, threshold)
    )
    metric = staticmethod(
        lambda name="evaluation_count", limit=lambda metric: metric.sum() > 1000: (
            MetricLimit(name, limit)
        )
    )


class Dist:
    euclidean = staticmethod(lambda: EuclideanDistance())
    cosine = staticmethod(lambda: CosineDistance())
    neat = staticmethod(
        lambda excess, disjoint, weight_diff: NeatDistance(
            excess, disjoint, weight_diff
        )
    )
    hamming = staticmethod(lambda: HammingDistance())
