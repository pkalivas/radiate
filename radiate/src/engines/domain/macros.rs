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
        for (name, metric) in $metric_set.iter().filter(|(_, m)| matches!(m, Metric::Value(_, _))) {
            if let Metric::Value(_, stat) = metric {
                println!(
                    "{:<20} | Mean: {:>8.3}, Min: {:>8.3}, Max: {:>8.3}, N: {:>3}",
                    name,
                    stat.mean(),
                    stat.min(),
                    stat.max(),
                    stat.count(),
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
macro_rules! log_ctx {
    ($ctx:expr) => {{
        let c = $ctx;
        println!(
            "[Iteration {:<4}] | Score: {:>8.4} | Pop. Size: {:>4} | Species: {:>3} | Elapsed: {:.2?}",
            c.index,
            c.score.as_ref().map(|s| s.as_f32()).unwrap_or_default(),
            c.population.len(),
            c.species.len(),
            c.timer.duration()
        );
    }};
}

#[macro_export]
macro_rules! metric {
    ($name:expr, $val:expr, $time:expr) => {{ Metric::new_operations($name, $val, $time) }};
    ($name:expr, $val:expr) => {{
        let mut metric = Metric::new_value($name);
        metric.add_value($val);
        metric
    }};
    ($name:expr, $dist:expr) => {{
        let mut metric = Metric::new_distribution($name)
        metric.add_distribution($dist);
        metric
    }};
    ($name:expr, $time:expr) => {{
        let mut metric = Metric::new_time($name);
        metric.add_time($time);
        metric
    }};
}
