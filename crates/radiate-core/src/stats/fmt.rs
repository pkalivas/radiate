use crate::stats::TagType;
use crate::{Metric, MetricSet, metric_names};
use radiate_utils::SmallStr;
use std::borrow::Cow;
use std::time::Duration;
use std::{fmt::Write as _, io};

const NAME_WIDTH: usize = 26;

fn truncate_name(name: &str) -> Cow<'_, str> {
    let len = name.chars().count();
    if len <= NAME_WIDTH {
        return Cow::Borrowed(name);
    }

    let available = NAME_WIDTH - 3;
    let head_len = available.div_ceil(2);
    let tail_len = available - head_len;
    let head = name.chars().take(head_len).collect::<String>();
    let tail = name.chars().skip(len - tail_len).collect::<String>();

    Cow::Owned(format!("{head}...{tail}"))
}

pub fn sparkline(values: &[f32], width: usize) -> String {
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

pub fn render_dashboard(metrics: &MetricSet) -> io::Result<String> {
    let mut out = String::new();

    let mut push_val = |name: &SmallStr, label: &str| {
        if let Some(m) = metrics.get(name) {
            let mu = m.mean();
            write!(out, "  {label}: {:.3}", mu).unwrap();
        }
    };

    push_val(&metric_names::CARRYOVER_RATE, "carryover");
    push_val(&metric_names::DIVERSITY_RATIO, "diversity");

    let mut push_int = |name: &SmallStr, label: &str| {
        if let Some(m) = metrics.get(name) {
            write!(out, "  {label}: {}", m.last_value() as i64).unwrap();
        }
    };

    push_int(&metric_names::UNIQUE_MEMBERS, "unique_members");
    push_int(&metric_names::UNIQUE_SCORES, "unique_scores");

    if let Some(m) = metrics.get(metric_names::BEST_SCORE_IMPROVEMENT) {
        write!(out, "  improvements: {}", m.count() as i64).unwrap();
    }

    if let Some(m) = metrics.get(metric_names::TIME)
        && let Some(mu) = m.times().map(|t| t.mean())
    {
        write!(out, "  iter_time(mean): {}", fmt_duration(mu)).unwrap();
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
        "{:<26} | {:<6} | {:<10} | {:<10} | {:<10} | {:<6} | {:<12} | {:<10} | {:<10} | {:<10}",
        "Name", "Type", "Mean", "Min", "Max", "N", "Total", "StdDev", "Skew", "Kurt"
    )
    .unwrap();
    writeln!(out, "{}", "-".repeat(135)).unwrap();
    Ok(out)
}

pub fn render_metric_rows_full(
    out: &mut String,
    name: &str,
    m: &Metric,
    tag: TagType,
) -> io::Result<()> {
    // Value row

    if let Some(dist) = m.distributions()
        && tag == TagType::Distribution
    {
        writeln!(
            out,
            "{:<26} | {:<6} | {:<10.3} | {:<10.3} | {:<10.3} | {:<6} | {:<12.3} | {:<10.3} | {:<10.3} | {:<10.3}",
            truncate_name(name),
            "dist",
            dist.mean(),
            dist.min(),
            dist.max(),
            dist.count(),
            dist.sum() / 1_000.0, // scale sum to avoid overflow
            dist.stddev(),
            dist.skewness(),
            dist.kurtosis()
            ,
        ).unwrap();
    }

    if let Some(stat) = m.stats()
        && tag == TagType::Statistic
    {
        writeln!(
            out,
            "{:<26} | {:<6} | {:<10.3} | {:<10.3} | {:<10.3} | {:<6} | {:<12.3} | {:<10.3} | {:<10.3} | {:<10.3}",
            truncate_name(name),
            "value",
            stat.mean(),
            stat.min(),
            stat.max(),
            stat.count(),
            stat.sum() / 1_000.0, // scale sum to avoid overflow
            stat.stddev(),
            stat.skewness(),
            stat.kurtosis(),
        ).unwrap();
    }

    // Time row
    if let Some(t) = m.times()
        && tag == TagType::Time
    {
        writeln!(
            out,
            "{:<26} | {:<6} | {:<10} | {:<10} | {:<10} | {:<6} | {:<12} | {:<10} | {:<10} | {:<10}",
            truncate_name(name),
            "time",
            fmt_duration(t.mean()),
            fmt_duration(t.min()),
            fmt_duration(t.max()),
            t.count(),
            fmt_duration(t.sum()),
            fmt_duration(t.stddev()),
            "-",
            "-",
        )
        .unwrap();
    }

    Ok(())
}

fn render_tagged(ms: &MetricSet, tag: TagType, title: &str) -> io::Result<String> {
    let mut out = String::new();
    writeln!(out, "== {} ==", title).unwrap();
    out = render_table_header(out)?;

    let mut items: Vec<_> = ms.iter_tagged(tag).collect();
    items.sort_by(|a, b| a.name().cmp(&b.name()));

    for m in items {
        render_metric_rows_full(&mut out, m.name().as_str(), m, tag)?;
    }

    Ok(out)
}

pub fn render_full(metrics: &MetricSet) -> io::Result<String> {
    let mut out = String::new();

    let summary = metrics.summary();
    let dash = render_dashboard(metrics)?;
    writeln!(
        out,
        "----- Metrics ----- ({} :: {}) \n{}",
        summary.metrics, summary.updates, dash
    )
    .unwrap();

    let dist = render_tagged(metrics, TagType::Distribution, "Distributions")?;
    writeln!(out, "\n{}", dist).unwrap();

    let generation = render_tagged(metrics, TagType::Statistic, "Statistics")?;
    writeln!(out, "\n{}", generation).unwrap();

    let life = render_tagged(metrics, TagType::Time, "Times")?;
    writeln!(out, "\n{}", life).unwrap();

    Ok(out)
}

pub fn fmt_duration(d: Duration) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_name_unchanged() {
        assert_eq!(truncate_name("count.species"), "count.species");
    }

    #[test]
    fn name_at_limit_unchanged() {
        let n: String = std::iter::repeat_n('a', NAME_WIDTH).collect();
        assert_eq!(truncate_name(&n), n.as_str());
    }

    #[test]
    fn long_name_gets_middle_ellipsis() {
        let out = truncate_name("mutate.graph.invalid.rejected");
        assert_eq!(out.chars().count(), NAME_WIDTH);
        assert!(out.contains("..."));
        assert!(out.starts_with("mutate"));
        assert!(out.ends_with("rejected"));
    }
}
