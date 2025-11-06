use std::time::Duration;
use std::{fmt::Write as _, io};

use crate::{Metric, MetricScope, MetricSet, metric_names};

/// ASCII sparkline for quick trend peeks (uses last distribution sequence).
/// Example: ▁▂▃▄▅▆▇█
fn sparkline(values: &[f32], width: usize) -> String {
    if values.is_empty() || width == 0 {
        return String::new();
    }

    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let (min, max) = values
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(mn, mx), &v| {
            (mn.min(v), mx.max(v))
        });

    let span = (max - min).max(1e-12);
    let step = (values.len() as f32 / width as f32).max(1.0);
    let mut out = String::with_capacity(width);

    let mut idx = 0.0;
    for _ in 0..width {
        let i = f32::floor(idx) as usize;
        let v = *values.get(i.min(values.len() - 1)).unwrap_or(&min);
        let level = (((v - min) / span) * ((blocks.len() - 1) as f32)).round() as usize;
        out.push(blocks[level.min(blocks.len() - 1)]);
        idx += step;
    }
    out
}

fn fmt_dur(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns == 0 {
        "0ns".into()
    } else if ns < 1_000 {
        format!("{ns}ns")
    } else if ns < 1_000_000 {
        format!("{:.3}µs", ns as f64 / 1e3)
    } else if ns < 1_000_000_000 {
        format!("{:.3}ms", ns as f64 / 1e6)
    } else {
        format!("{:.3}s", ns as f64 / 1e9)
    }
}

pub fn render_dashboard(metrics: &MetricSet) -> io::Result<String> {
    let mut out = String::new();

    let mut push_val = |name: &'static str, label: &str| {
        if let Some(m) = metrics.get(name) {
            if let Some(mu) = m.value_mean() {
                write!(out, "  {label}: {:.3}", mu).unwrap();
            } else if m.count() > 0 {
                write!(out, "  {label}: {:.3}", m.last_value()).unwrap();
            }
        }
    };

    push_val(metric_names::CARRYOVER_RATE, "carryover");
    push_val(metric_names::DIVERSITY_RATIO, "diversity");

    let mut push_int = |name: &'static str, label: &str| {
        if let Some(m) = metrics.get(name) {
            write!(out, "  {label}: {}", m.last_value() as i64).unwrap();
        }
    };

    push_int(metric_names::UNIQUE_MEMBERS, "unique_members");
    push_int(metric_names::UNIQUE_SCORES, "unique_scores");

    if let Some(m) = metrics.get(metric_names::LIFETIME_UNIQUE_MEMBERS) {
        write!(out, "  lifetime_unique: {}", m.last_value() as i64).unwrap();
    }

    if let Some(m) = metrics.get(metric_names::TIME) {
        if let Some(mu) = m.time_mean() {
            write!(out, "  iter_time(mean): {}", fmt_dur(mu)).unwrap();
        }
    }

    Ok(if out.is_empty() {
        "—".into()
    } else {
        out.replace('\n', "")
    })
}

fn render_table_header(mut out: String) -> io::Result<String> {
    writeln!(
        out,
        "{:<24} | {:<6} | {:<10} | {:<10} | {:<10} | {:<6} | {:<12} | {:<10} | {:<10} | {:<10} | {:<10}",
        "Name", "Type", "Mean", "Min", "Max", "N", "Total", "StdDev", "Skew", "Kurt", "Entr"
    ).unwrap();
    writeln!(out, "{}", "-".repeat(145)).unwrap();
    Ok(out)
}

pub fn render_metric_rows(
    out: &mut String,
    name: &str,
    m: &Metric,
    include_spark: bool,
) -> io::Result<()> {
    let inner = m.inner();

    if name == super::set::METRIC_SET {
        if let Some(s) = m.statistic() {
            writeln!(
                out,
                "Metric Set [metrics: {}, updates: {:.0}]",
                s.sum(),
                s.sum()
            )
            .unwrap();
        }
    }

    // Value row
    if let Some(stat) = m.statistic() {
        writeln!(
            out,
            "{:<24} | {:<6} | {:<10.3} | {:<10.3} | {:<10.3} | {:<6} | {:<12} | {:<10.3} | {:<10.3} | {:<10.3} | {:<10}",
            name,
            "value",
            stat.mean(),
            stat.min(),
            stat.max(),
            stat.count(),
            "-",
            stat.std_dev(),
            stat.skewness(),
            stat.kurtosis(),
            "-",
        ).unwrap();
    }

    // Time row
    if let Some(t) = m.time_statistic() {
        writeln!(
            out,
            "{:<24} | {:<6} | {:<10} | {:<10} | {:<10} | {:<6} | {:<12} | {:<10} | {:<10} | {:<10} | {:<10}",
            name,
            "time",
            fmt_dur(t.mean()),
            fmt_dur(t.min()),
            fmt_dur(t.max()),
            t.count(),
            fmt_dur(t.sum()),
            fmt_dur(t.standard_deviation()),
            "-",
            "-",
            "-",
        ).unwrap();
    }

    // Distribution row (+ optional sparkline)
    if let Some(dist) = inner.distribution.as_ref() {
        writeln!(
            out,
            "{:<24} | {:<6} | {:<10.3} | {:<10.3} | {:<10.3} | {:<6} | {:<12} | {:<10.3} | {:<10.3} | {:<10.3} | {:<10.3}",
            name,
            "dist",
            dist.mean(),
            dist.min(),
            dist.max(),
            dist.count(),
            format!("{:.3}", dist.entropy()),
            dist.standard_deviation(),
            dist.skewness(),
            dist.kurtosis(),
            format!("{:.3}", dist.entropy()),
        ).unwrap();

        if include_spark {
            if let Some(seq) = m.last_sequence() {
                if !seq.is_empty() {
                    let s = sparkline(seq, 40);
                    writeln!(out, "{:<24}   {}", "", s).unwrap();
                }
            }
        }
    }

    Ok(())
}

fn render_scope(
    ms: &MetricSet,
    scope: MetricScope,
    title: &str,
    include_spark: bool,
) -> io::Result<String> {
    let mut out = String::new();
    writeln!(out, "== {} ==", title).unwrap();
    out = render_table_header(out)?;

    let mut items: Vec<_> = ms.iter_scope(scope).collect();
    items.sort_by(|a, b| a.0.cmp(b.0));

    for (name, m) in items {
        render_metric_rows(&mut out, name, m, include_spark)?;
    }

    Ok(out)
}

/// Render a full, multi-scope view:
/// - dashboard summary line
/// - Generation table
/// - Lifetime table
/// - Step timings table
pub fn render_full(metrics: &MetricSet) -> io::Result<String> {
    let mut out = String::new();

    let dash = render_dashboard(metrics)?;
    writeln!(out, "[metrics]{}", dash).unwrap();

    let generation = render_scope(metrics, MetricScope::Generation, "Generation", true)?;
    writeln!(out, "\n{}", generation).unwrap();

    let life = render_scope(metrics, MetricScope::Lifetime, "Lifetime", false)?;
    writeln!(out, "\n{}", life).unwrap();

    let step = render_scope(metrics, MetricScope::Step, "Step Timings", false)?;
    writeln!(out, "\n{}", step).unwrap();

    Ok(out)
}
