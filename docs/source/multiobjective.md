!!! warning ":construction: Under Construction :construction:"

    These docs are a work in progress and may not be complete or accurate. Please check back later for updates.

# Multiobjective Optimization

The goal of MultiObjective optimization is to find a set of solutions that are optimal with respect to all objectives. These solutions are called Pareto optimal solutions while the set of all Pareto optimal solutions is called the Pareto front.

`radiate` supports this type of optimization through simply supplying a list of objectives instead of a single one. This also means the result of the engine's `fitness_fn` must be a list of values equal to the number of objectives. The pareto front will collect as many solutions as specified by the `front_size` parameter, which is a range of values. If the number of solutions collected is greater than the upper bound of the range, the front will filter out the solutions based on their pareto dominance to ensure only the best solutions are kept.

## DTLZ 1

The DTLZ1 problem is a well-known multiobjective optimization problem that is used to test the performance of multiobjective optimization algorithms. It is a 3-objective problem with 4 variables and is defined as:

$$
\begin{align*}
\text{minimize} \quad & f_1(x) = (1 + g) \cdot x_1 \cdot x_2 \\
\text{minimize} \quad & f_2(x) = (1 + g) \cdot x_1 \cdot (1 - x_2) \\
\text{minimize} \quad & f_3(x) = (1 + g) \cdot (1 - x_1) \\
\text{subject to} \quad & 0 \leq x_i \leq 1 \quad \text{for} \quad i = 1, 2, 3, 4 \\
\text{where} \quad & g = \sum_{i=3}^{4} (x_i - 0.5)^2
\end{align*}
$$

To set up the engine to solve this type of problem, we supply the objective functions as a list of `Optimize` enums. The `Optimize` enum is used to specify whether the objective should be minimized, maximized, or any combination of the two. In this case, we want to minimize all objectives.

!!! code "DTLZ1"

    ``` rust 
    use radiate::*;

    const VARIABLES: usize = 4;
    const OBJECTIVES: usize = 3;
    const K: usize = VARIABLES - OBJECTIVES + 1;

    fn main() {
        let codec = FloatCodec::vector(VARIABLES, 0_f32..1_f32).with_bounds(-100.0, 100.0);

        let engine = GeneticEngine::builder()
            .codec(codec)
            .num_threads(10)
            .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
            .front_size(1100..1300)
            .offspring_selector(RouletteSelector::new())
            .survivor_selector(NSGA2Selector::new())
            .alter(alters!(
                SimulatedBinaryCrossover::new(1_f32, 1.0),
                UniformMutator::new(0.1_f32),
            ))
            .fitness_fn(|geno: Vec<f32>| {
                let values = &geno;
                let g = values[K..]
                    .iter()
                    .map(|&xi| (xi - 0.5).powi(2))
                    .sum::<f32>();

                let f1 = (1.0 + g) * (values[0] * values[1]);
                let f2 = (1.0 + g) * (values[0] * (1.0 - values[1]));
                let f3 = (1.0 + g) * (1.0 - values[0]);

                vec![f1, f2, f3]
            })
            .build();

        let result = engine.run(|ctx| {
            println!("[ {:?} ]", ctx.index);
            ctx.index > 1000
        });

        // typically we can get the 'best' solution from the result by calling
        // 'result.best', but in a multiobjective case we only care about the Pareto front
        // which is stored in 'result.front' and can be accessed like so:
        let front = result.front;
    }
    ```

The resulting Pareto front can be seen below:

<div id="dtlz_1"></div>


## DTLZ 2

The DTLZ2 problem is another well-known multiobjective optimization problem that is used to test the performance of multiobjective optimization algorithms. It is a 3-objective problem with 4 variables and is defined as:

$$
\begin{align*}
\text{minimize} \quad & f_1(x) = (1 + g) \cdot \cos(x_1 \cdot \pi / 2) \cdot \cos(x_2 \cdot \pi / 2) \\
\text{minimize} \quad & f_2(x) = (1 + g) \cdot \cos(x_1 \cdot \pi / 2) \cdot \sin(x_2 \cdot \pi / 2) \\
\text{minimize} \quad & f_3(x) = (1 + g) \cdot \sin(x_1 \cdot \pi / 2) \\
\text{subject to} \quad & 0 \leq x_i \leq 1 \quad \text{for} \quad i = 1, 2, 3, 4 \\
\text{where} \quad & g = \sum_{i=3}^{4} (x_i - 0.5)^2
\end{align*}
$$

Again, to set up the engine to solve this type of problem, we supply the objective functions as a list of `Optimize` enums. In this case, we want to minimize all objectives.

!!! code "DTLZ2"

    ``` rust 
    use radiate::*;

    const VARIABLES: usize = 4;
    const OBJECTIVES: usize = 3;
    const K: usize = VARIABLES - OBJECTIVES + 1;

    fn main() {
        let codec = FloatCodec::vector(VARIABLES, 0_f32..1_f32).with_bounds(-100.0, 100.0);

        let engine = GeneticEngine::builder()
            .codec(codec)
            .num_threads(10)
            .multi_objective(vec![Optimize::Minimize; OBJECTIVES])
            .front_size(1000..1100)
            .survivor_selector(NSGA2Selector::new())
            .alter(alters!(
                SimulatedBinaryCrossover::new(1_f32, 1.0),
                UniformMutator::new(0.1_f32),
            ))
            .fitness_fn(|geno: Vec<f32>| {
                let values = &geno;
                let g = values[K..]
                    .iter()
                    .map(|&xi| (xi - 0.5).powi(2))
                    .sum::<f32>();

                let mut f = vec![1.0 + g; OBJECTIVES];
                for i in 0..OBJECTIVES {
                    for j in 0..OBJECTIVES - (i + 1) {
                        f[i] *= (values[j] * 0.5 * std::f32::consts::PI).cos();
                    }
                    if i != 0 {
                        let aux = OBJECTIVES - (i + 1);
                        f[i] *= (values[aux] * 0.5 * std::f32::consts::PI).sin();
                    }
                }

                f
            })
            .build();

        let result = engine.run(|ctx| {
            println!("[ {:?} ]", ctx.index);
            ctx.index > 1000
        });

        // typically we can get the 'best' solution from the result by calling
        // 'result.best', but in a multiobjective case we only care about the Pareto front
        // which is stored in 'result.front' and can be accessed like so:
        let front = result.front;
    }
    ```

The resulting Pareto front can be seen below:

<div id="dtlz_2"></div>

<script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
<script>
Promise.all([
    fetch("../../assets/dtlz_1.json").then(response => response.json()),
    fetch("../../assets/dtlz_2.json").then(response => response.json())
])
.then(([dtlz1, dtlz2]) => {
    let x1 = [], y1 = [], z1 = [];
    let x2 = [], y2 = [], z2 = [];

    dtlz1.pareto_front.forEach(point => {
        x1.push(point[0]);
        y1.push(point[1]);
        z1.push(point[2]);
    });

    dtlz2.pareto_front.forEach(point => {
        x2.push(point[0]);
        y2.push(point[1]);
        z2.push(point[2]);
    });

    let trace1 = {
        x: x1,
        y: y1,
        z: z1,
        mode: "markers",
        type: "scatter3d",
        name: "DTLZ1",
        marker: { size: 5, color: "blue" }
    };

    let trace2 = {
        x: x2,
        y: y2,
        z: z2,
        mode: "markers",
        type: "scatter3d",
        name: "DTLZ2",
        marker: { size: 5, color: "red" }
    };

    Plotly.newPlot("dtlz_1", [trace1]);
    Plotly.newPlot("dtlz_2", [trace2]);
})
.catch(error => console.error(error));
</script>