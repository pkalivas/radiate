#[macro_export]
macro_rules! impl_integer {
    ($($t:ty),*) => {
        $(
            impl Integer<$t> for $t {
                const MIN: $t = <$t>::MIN;
                const MAX: $t = <$t>::MAX;

                fn from_i32(value: i32) -> $t {
                    value as $t
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! alters {
    ($($struct_instance:expr),* $(,)?) => {
        {
            let mut vec: Vec<Box<dyn Alter<_>>> = Vec::new();
            $(
                vec.push(Box::new($struct_instance.alterer()));
            )*
            vec
        }
    };
}

#[macro_export]
macro_rules! filter_alters {
    ($chromosome_type:ty; $($input:expr),* $(,)?) => {{
        trait IsCompatibleWithChromosomeType {}
        impl<T: Alter<$chromosome_type>> IsCompatibleWithChromosomeType for T {}

        let mut vec: Vec<Box<dyn Alter<$chromosome_type>>> = Vec::new();

        vec
    }};
}

#[macro_export]
macro_rules! bench {
    ($name:literal, $operation:expr) => {
        let timer = std::time::Instant::now();
        let result = $operation;
        let elapsed = timer.elapsed();
        println!("{:?} took {:?}", $name, elapsed);
        result
    };
}

#[macro_export]
macro_rules! print_metrics {
    ($metric_set:expr, [$($filter:expr),* $(,)?]) => {{
        use std::collections::HashSet;
        let filter_set: HashSet<&str> = vec![$($filter),*].into_iter().collect();

        println!("=================================================== Metrics Summary ====================================================");

        // Display in order: Operations, Values, Distributions, Times
        for metric_type in ["Operations", "Value", "Distribution", "Time"] {
            for (name, metric) in $metric_set.iter() {
                if !filter_set.contains(name) {
                    continue;
                }
                match (metric_type, metric) {
                    ("Operations", Metric::Operations(_, _, _)) => println!("{:?}", metric),
                    ("Value", Metric::Value(_, _)) => println!("{:?}", metric),
                    ("Distribution", Metric::Distribution(_, _)) => println!("{:?}", metric),
                    ("Time", Metric::Time(_, _)) => println!("{:?}", metric),
                    _ => {},
                }
            }
        }
        println!("========================================================================================================================");
    }};
    ($metric_set:expr) => {{
        use std::time::Duration;

        println!("=================================================== Metrics Summary ====================================================");

        // Operations first
        for (name, metric) in $metric_set.iter().filter(|(_, m)| matches!(m, Metric::Operations(_, _, _))) {
            if let Metric::Operations(_, stat, time_stat) = metric {
                println!(
                    "{:<20} | Mean: {:>8.3}, Min: {:>8.3}, Max: {:>8.3}, N: {:>3} | Avg Time: {:>9.3?}, Total Time: {:>9.3?}",
                    name,
                    stat.mean(),
                    stat.min(),
                    stat.max(),
                    stat.count(),
                    time_stat.mean(),
                    time_stat.sum(),
                );
            }
        }

        // Values next
        for (name, metric) in $metric_set.iter().filter(|(_, m)| matches!(m, Metric::Value(_, _, _))) {
            if let Metric::Value(_, stat, dist) = metric {
                println!(
                    "{:<20} | Mean: {:>8.3}, Min: {:>8.3}, Max: {:>8.3}, N: {:>3} | Dist. Mean: {:>8.3}, Dist. StdDev: {:>8.3}, Dist. Min: {:>8.3}, Dist. Max: {:>8.3}",
                    name,
                    stat.mean(),
                    stat.min(),
                    stat.max(),
                    stat.count(),
                    dist.mean(),
                    dist.standard_deviation(),
                    dist.min(),
                    dist.max(),

                );
            }
        }

        // Distributions next
        for (name, metric) in $metric_set.iter().filter(|(_, m)| matches!(m, Metric::Distribution(_, _))) {
            if let Metric::Distribution(_, dist) = metric {
                println!(
                    "{:<20} | Mean: {:>8.3}, StdDev: {:>8.3}, Min: {:>8.3}, Max: {:>8.3}, N: {:>3}",
                    name,
                    dist.mean(),
                    dist.standard_deviation(),
                    dist.min(),
                    dist.max(),
                    dist.count(),
                );
            }
        }

        // Times last
        for (name, metric) in $metric_set.iter().filter(|(_, m)| matches!(m, Metric::Time(_, _))) {
            if let Metric::Time(_, stat) = metric {
                println!(
                    "{:<20} | Avg Time: {:>9.3?}, Min Time: {:>9.3?}, Max Time: {:>9.3?}, N: {:>3} | Total Time: {:>9.3?}",
                    name,
                    stat.mean(),
                    stat.min(),
                    stat.max(),
                    stat.count(),
                    stat.sum(),
                );
            }
        }

        println!("========================================================================================================================");
    }};
}

#[macro_export]
macro_rules! metricset_to_string {
    ($metric_set:expr) => {{
        use std::collections::HashSet;
        let mut result = String::new();

        for (name, metric) in $metric_set.iter() {
            result.push_str(&format!("{:?}\n", metric));
        }
        result
    }};
}

#[macro_export]
macro_rules! log_ctx {
    ($ctx:expr) => {{
        println!(
            "[ Iteration {:<4} ] Score: {:>8.4}, Elapsed: {:.2?}",
            $ctx.index(),
            $ctx.score().as_f32(),
            $ctx.time()
        );
    }};
}

#[macro_export]
macro_rules! histogram {
    ($title:expr, $data:expr) => {{
        let max = $data.iter().cloned().fold(f32::MIN, f32::max);
        let min = $data.iter().cloned().fold(f32::MAX, f32::min);
        let bins = 10;
        let step = (max - min) / bins as f32;
        for i in 0..bins {
            let lower = min + i as f32 * step;
            let upper = lower + step;
            let count = $data.iter().filter(|&&x| x >= lower && x < upper).count();
            println!("  {:6.2} - {:6.2}: {}", lower, upper, "█".repeat(count));
        }
    }};
}

#[macro_export]
macro_rules! dbg_ctx {
    ($val:expr $(,)?) => {{
        let tmp = &$val;
        println!("[{}:{}] {} = {:?}", file!(), line!(), stringify!($val), tmp);
        tmp
    }};
}

#[macro_export]
macro_rules! build_engine {
    (
        codec: $codec:expr,
        fitness: $fitness_fn:expr,
        settings: { $( $setting:ident $( : $value:expr )? ),* $(,)? }
    ) => {{
        let builder = GeneticEngine::builder().codec($codec).fitness_fn($fitness_fn);
        $(
            #[allow(unused_mut)]
            let builder = crate::build_engine!(@apply_setting builder, $setting $(, $value)?);
        )*
        builder.build()
    }};

    // helper to apply each setting appropriately
    (@apply_setting $builder:ident, $method:ident, $value:expr) => {
        $builder.$method($value)
    };
    (@apply_setting $builder:ident, $method:ident) => {
        $builder.$method()
    };
}

#[macro_export]
macro_rules! engine {
    ($codec:expr, $fitness:expr) => {
        GeneticEngine::builder().codec($codec).fitness_fn($fitness).build()
    };
    ($codec:expr, $fitness:expr, $($extra:tt)+) => {
        GeneticEngine::builder().codec($codec).fitness_fn($fitness).$($extra)+.build()
    };
}

#[macro_export]
macro_rules! experiment {
    (
        repeat: $reps:expr,
        $codec:expr,
        $fitness:expr,
        [$( $setting:ident ( $($value:expr),* ) ),* $(,)?],
        $condition:expr
    ) => {
        (0..$reps)
            .map(|_| {
                let engine = GeneticEngine::builder()
                    .codec($codec)
                    .fitness_fn($fitness)
                    $( .$setting($($value),*) )*
                    .build();
                engine.run($condition)
            })
            .collect::<Vec<_>>()
    };
}

// let results = experiment!(
//     repeat: 10,
//     FloatCodec::vector(5, -10.0..10.0),
//     |geno: Vec<f32>| geno.iter().sum::<f32>(),
//     [
//         minimizing(),
//         population_size(200),
//         num_threads(4),
//         alter(alters!(SwapMutator::new(0.05), UniformCrossover::new(0.5)))
//     ],
//     |ctx| ctx.score().as_f32() < 0.01
// );

// MACRO IDEAS

// #[macro_export]
// macro_rules! genetic_test {
//     (
//         name: $name:ident,
//         codec: $codec:expr,
//         fitness: $fitness_fn:expr,
//         settings: { $( $setting:ident $( : $value:expr )? ),* $(,)? },
//         stopping_criteria: |$ctx:ident| $criteria:expr,
//         assert: |$result:ident| $assertion:block
//     ) => {
//         #[test]
//         fn $name() {
//             let engine = crate::build_engine!(
//                 codec: $codec,
//                 fitness: $fitness_fn,
//                 settings: { $($setting $( : $value )?),* }
//             );

//             let $result = engine.run(|$ctx| $criteria);

//             $assertion
//         }
//     };
// }

// genetic_test!(
//     name: evolve_zero_vector,
//     codec: FloatCodec::vector(5, -10.0..10.0),
//     fitness: |geno| geno.iter().map(|x| x * x).sum::<f32>(),
//     settings: {
//         minimizing,
//         population_size: 50,
//         num_threads: 4,
//     },
//     stopping_criteria: |ctx| {
//         // Stop when the score is close to zero
//         ctx.score().as_f32() < 0.01
//     },
//     assert: |result| {
//         assert!(result.score().as_f32() < 0.1);
//     }
// );
