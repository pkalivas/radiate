//! ascii_dashboard.rs
//!
//! Live ASCII dashboard for Radiate, "classic" style.
//!
//! Usage sketch:
//!
//! let mut dash = AsciiDashboard::new(std::time::Duration::from_millis(100));
//! for gen in 0..max_gens {
//!     engine.step(&mut metrics)?;
//!     dash.maybe_render(gen, &metrics)?;
//! }

use std::collections::HashSet;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::stats::defaults::metric_tags;
use crate::{MetricScope, MetricSet};

const WIDTH: usize = 79;
const HLINE: &str =
    "-------------------------------------------------------------------------------";

pub struct AsciiDashboard {
    refresh_interval: Duration,
    last_render: Instant,
}

impl AsciiDashboard {
    /// Create a new dashboard with a minimum refresh interval.
    ///
    /// Example: `AsciiDashboard::new(Duration::from_millis(100))`
    pub fn new(refresh_interval: Duration) -> Self {
        let now = Instant::now();
        Self {
            refresh_interval,
            last_render: now - refresh_interval,
        }
    }

    /// Force a full render, ignoring `refresh_interval`.
    pub fn render_now(&mut self, generation: usize, metrics: &MetricSet) -> io::Result<()> {
        self.last_render = Instant::now();
        render_ascii_dashboard(generation, metrics)
    }

    /// Render only if enough time has passed since the last render.
    pub fn maybe_render(&mut self, generation: usize, metrics: &MetricSet) -> io::Result<()> {
        if self.last_render.elapsed() >= self.refresh_interval {
            self.render_now(generation, metrics)?;
        }
        Ok(())
    }
}

/// Clear terminal and print the classic dashboard for this generation.
pub fn render_ascii_dashboard(generation: usize, metrics: &MetricSet) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Clear screen + move cursor to top-left (ANSI escape codes).
    write!(stdout, "\x1B[2J\x1B[H")?;

    // Derive an elapsed time from your lifetime "time" metric if present.
    let elapsed = metrics
        .time()
        .and_then(|m| m.time_sum())
        .unwrap_or_default();

    let text = render_classic_dashboard(generation, elapsed, metrics);
    write!(stdout, "{text}")?;
    stdout.flush()?;
    Ok(())
}

/// Core formatter: returns the full classic dashboard as a String.
///
/// Layout:
/// -------------------------------------------------------------------------------
/// | 1392                     : Minimum Fitness=0.04638               00:05:0000 |
/// -------------------------------------------------------------------------------
/// | Obj 0             ∧=2.9954     ∨=0.0464      μ=0.1871   s²=0.149 S=6.125    |
/// -------------------------------------------------------------------------------
/// | Evaluate          Σ=65561    00:01:7270      μ=47.0646  00:00:0010          |
/// | ...                                                                      ...|
/// -------------------------------------------------------------------------------
/// | Age               ∧=65.0000    ∨=0.0000      μ=2.6339                       |
/// | Unique            ∧=75.0000    ∨=1.0000      μ=53.4217                      |
/// | ...                                                                      ...|
/// -------------------------------------------------------------------------------
pub fn render_classic_dashboard(
    generation: usize,
    elapsed: Duration,
    metrics: &MetricSet,
) -> String {
    let mut out = String::new();

    // --- HEADER ---------------------------------------------------------------
    let min_fitness = metrics
        .score()
        .and_then(|m| m.distribution_min())
        .or_else(|| metrics.score().and_then(|m| m.value_min()))
        .unwrap_or(0.0);

    let elapsed_str = fmt_duration(elapsed);

    // Header line: generation, min fitness, total time
    // | 1392                     : Minimum Fitness=0.04638               00:05:0000 |
    out.push_str(HLINE);
    out.push('\n');
    out.push('|');

    // Left: generation
    let gen_str = format!(" {}", generation);
    out.push_str(&pad_right(&gen_str, 24));

    // Middle: minimum fitness
    let min_str = format!(" Fitness={:.5}", min_fitness);
    out.push_str(&pad_right(&min_str, 40)); // 24 + 40 = 64 chars inside

    // Right: elapsed time, right-aligned within remaining width
    let remaining = WIDTH - 2 - 24 - 40; // minus borders and previous fields
    out.push_str(&pad_left(&elapsed_str, remaining));

    out.push_str("|\n");
    out.push_str(HLINE);
    out.push('\n');

    // --- OBJECTIVE SUMMARY ----------------------------------------------------
    if let Some(score) = metrics.score() {
        if let Some(stat) = score.statistic() {
            // | Obj 0             ∧=2.9954     ∨=0.0464      μ=0.1871   s²=0.149 S=6.125    |
            let name = "Obj 0";
            let max = stat.max();
            let min = stat.min();
            let mean = stat.mean();
            let var = stat.variance();
            let skew = stat.skewness();

            out.push('|');
            out.push_str(&pad_right(name, 19)); // "Obj 0" + padding

            let segment = format!(
                "∧={max:<7.4}  ∨={min:<7.4}   μ={mean:<7.4}  s²={var:<5.3} S={skew:<6.3}",
                max = max,
                min = min,
                mean = mean,
                var = var,
                skew = skew,
            );
            out.push_str(&pad_right(&segment, WIDTH - 2 - 19));
            out.push_str("|\n");
            out.push_str(HLINE);
            out.push('\n');
        }
    }

    // --- STEP METRICS ---------------------------------------------------------
    // We show each Step-scope metric on one line, in the style:
    // | Evaluate          Σ=65561    00:01:7270      μ=47.0646  00:00:0010          |
    let mut seen = HashSet::new();
    render_step_metrics(metrics, &mut out, &mut seen);
    // --- SELECTOR METRICS -----------------------------------------------------
    // Show Selector-scope metrics in compact min/max/mean form:
    // | SelectorName      ∨=10ms      ∧=50ms       Σ=500ms     μ=25ms            |
    render_selector_metrics(metrics, &mut out, &mut seen);
    // --- ALTERER METRICS ------------------------------------------------------
    // Show Alterer-scope metrics in compact sum/mean form:
    // | X CrossoverName  Σ=1500      00:01:500   μ=15.0000  00:00:150         |
    render_alterer_metrics(metrics, &mut out, &mut seen);

    // --- GENERATION (POPULATION) METRICS --------------------------------------
    // Show Generation-scope metrics in compact min/max/mean form:
    // | Age               ∧=65.0000    ∨=0.0000      μ=2.6339                       |
    let mut gen_items: Vec<_> = metrics.iter_scope(MetricScope::Generation).collect();
    gen_items.sort_by(|a, b| a.0.cmp(b.0));

    for (name, m) in gen_items {
        if seen.contains(name) {
            continue;
        }
        if let Some(stat) = m.statistic() {
            let max = stat.max();
            let min = stat.min();
            let mean = stat.mean();

            out.push('|');
            out.push_str(&pad_right(name, 16));

            let body = format!(
                " ∧={max:<8.4} ∨={min:<8.4} μ={mean:<8.4}",
                max = max,
                min = min,
                mean = mean,
            );

            out.push_str(&pad_right(&body, WIDTH - 2 - 16));
            out.push_str("|\n");
        }
    }

    out.push_str(HLINE);
    out.push('\n');

    out
}

fn render_step_metrics(metrics: &MetricSet, out: &mut String, seen: &mut HashSet<&'static str>) {
    let mut step_items = metrics.iter_scope(MetricScope::Step).collect::<Vec<_>>();
    step_items.sort_by(|a, b| a.0.cmp(b.0));

    for (name, m) in step_items {
        seen.insert(name);

        let min_time = m.time_min().map(fmt_duration).unwrap_or_default();
        let max_time = m.time_max().map(fmt_duration).unwrap_or_default();
        let total_time = m.time_sum().map(fmt_duration).unwrap_or_default();
        let mean_time = m.time_mean().map(fmt_duration).unwrap_or_default();

        out.push('|');

        // Name column (left)
        let name = name
            .split_once("_step")
            .map(|(n, _)| n.trim())
            .unwrap_or(name);
        out.push_str(&pad_right(name, 16));

        // Value + time columns
        let body =
            format!("∨={min_time:<10} ∧={max_time:<10} Σ={total_time:<10} μ={mean_time:<10}");
        out.push_str(&pad_right(&body, WIDTH - 2 - 16));
        out.push_str("|\n");
    }

    out.push_str(HLINE);
    out.push('\n');
}

fn render_selector_metrics(
    metrics: &MetricSet,
    out: &mut String,
    seen: &mut HashSet<&'static str>,
) {
    let mut selector_items = metrics
        .iter_tagged(metric_tags::SELECTOR)
        .collect::<Vec<_>>();
    selector_items.sort_by(|a, b| a.0.cmp(b.0));

    for (name, m) in selector_items {
        seen.insert(name);

        let max_time = m.time_max().map(fmt_duration).unwrap_or_default();
        let min_time = m.time_min().map(fmt_duration).unwrap_or_default();
        let mean_time = m.time_mean().map(fmt_duration).unwrap_or_default();
        let total_time = m.time_sum().map(fmt_duration).unwrap_or_default();

        out.push('|');
        let name = name
            .split_once("_selector")
            .map(|(n, _)| n.trim())
            .unwrap_or(name);
        out.push_str(&pad_right(name, 16));

        let body =
            format!("∨={min_time:<10} ∧={max_time:<10} Σ={total_time:<10} μ={mean_time:<10}");

        out.push_str(&pad_right(&body, WIDTH - 2 - 16));
        out.push_str("|\n");
    }

    out.push_str(HLINE);
    out.push('\n');
}

fn render_alterer_metrics(metrics: &MetricSet, out: &mut String, seen: &mut HashSet<&'static str>) {
    let mut alters = metrics
        .iter_tagged(metric_tags::ALTERER)
        .collect::<Vec<_>>();
    alters.sort_by(|a, b| a.0.cmp(b.0));

    for (name, m) in alters {
        seen.insert(name);

        let total_value = m
            .value_sum()
            .map(|v| format!("{:.0}", v))
            .unwrap_or_default();
        let value_mean = m
            .value_mean()
            .map(|v| format!("{:.4}", v))
            .unwrap_or_default();

        let mean_time = m.time_mean().map(fmt_duration).unwrap_or_default();
        let total_time = m.time_sum().map(fmt_duration).unwrap_or_default();

        out.push('|');
        let name = match name {
            n if n.contains("crossover") => n
                .split_once("_crossover")
                .map(|(n, _)| format!("X {}", n.trim()))
                .unwrap_or(name.to_string()),
            n if n.contains("mutator") => n
                .split_once("_mutator")
                .map(|(n, _)| format!("M {}", n.trim()))
                .unwrap_or(name.to_string()),
            _ => name.to_string(),
        };

        out.push_str(&pad_right(&name, 16));

        let body =
            format!("Σ={total_value:<10} {total_time:<10} μ={value_mean:<10} {mean_time:<10}");

        out.push_str(&pad_right(&body, WIDTH - 2 - 16));
        out.push_str("|\n");
    }

    out.push_str(HLINE);
    out.push('\n');
}

/// Simple right-padding helper to fit into a fixed-width cell.
fn pad_right(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        s.chars().take(width).collect()
    } else {
        let mut out = String::with_capacity(width);
        out.push_str(s);
        for _ in 0..(width - len) {
            out.push(' ');
        }
        out
    }
}

/// Simple left-padding helper (right-align within a fixed-width cell).
fn pad_left(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        s.chars().take(width).collect()
    } else {
        let mut out = String::with_capacity(width);
        for _ in 0..(width - len) {
            out.push(' ');
        }
        out.push_str(s);
        out
    }
}

/// Duration formatter similar to your existing `fmt_dur`, but local here.
fn fmt_duration(d: Duration) -> String {
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
