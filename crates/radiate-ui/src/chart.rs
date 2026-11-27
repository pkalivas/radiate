use radiate_utils::WindowBuffer;
use ratatui::style::Color;

/// How much history the line charts show, cycled with `z`.
const HISTORY_STEPS: [History; 3] = [History::All, History::Last(500), History::Last(100)];

/// How every line chart is drawn: the y scale and how much history is shown.
/// One setting for all charts, held in the nav state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChartView {
    pub scale: Scale,
    pub history: History,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scale {
    #[default]
    Linear,
    /// `log10` when every visible value is positive. Minimising runs often
    /// reach zero or go negative, so otherwise it's a symmetric log,
    /// `sign(y) * log10(1 + |y|)`, which handles both.
    Log,
}

/// The y mapping one render uses, chosen from the view and the visible data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transform {
    Identity,
    Log10,
    SymLog,
}

impl Transform {
    pub fn to_chart(self, y: f64) -> f64 {
        match self {
            Transform::Identity => y,
            Transform::Log10 => y.log10(),
            Transform::SymLog => y.signum() * y.abs().ln_1p() / std::f64::consts::LN_10,
        }
    }

    /// The real value at chart coordinate `y`, for axis labels.
    pub fn to_value(self, y: f64) -> f64 {
        match self {
            Transform::Identity => y,
            Transform::Log10 => 10f64.powf(y),
            Transform::SymLog => y.signum() * (10f64.powf(y.abs()) - 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum History {
    #[default]
    All,
    Last(usize),
}

impl ChartView {
    pub fn toggle_scale(&mut self) {
        self.scale = match self.scale {
            Scale::Linear => Scale::Log,
            Scale::Log => Scale::Linear,
        };
    }

    pub fn next_history(&mut self) {
        let i = HISTORY_STEPS
            .iter()
            .position(|&h| h == self.history)
            .unwrap_or(0);
        self.history = HISTORY_STEPS[(i + 1) % HISTORY_STEPS.len()];
    }

    /// The newest points of `values` this view shows.
    pub fn visible<'a>(&self, values: &'a [(f64, f64)]) -> &'a [(f64, f64)] {
        match self.history {
            History::All => values,
            History::Last(n) => &values[values.len().saturating_sub(n)..],
        }
    }

    /// The transform for drawing `ys`, the visible values of every series.
    pub fn transform(&self, mut ys: impl Iterator<Item = f64>) -> Transform {
        match self.scale {
            Scale::Linear => Transform::Identity,
            Scale::Log if ys.all(|y| y > 0.0) => Transform::Log10,
            Scale::Log => Transform::SymLog,
        }
    }

    /// A short description for the chart border; `None` for the default view.
    pub fn label(&self) -> Option<String> {
        let mut parts = Vec::new();
        if self.scale == Scale::Log {
            parts.push("log".to_string());
        }
        if let History::Last(n) = self.history {
            parts.push(format!("last {n}"));
        }
        (!parts.is_empty()).then(|| format!(" {} ", parts.join(" · ")))
    }
}

pub trait LineChart {
    fn values(&self) -> &[(f64, f64)];
    fn color(&self) -> Color;
    fn title(&self) -> &str;
}

pub struct RollingLineChart {
    title: String,
    values: WindowBuffer<(f64, f64)>,
    color: Color,
    point_count: usize,
}

impl RollingLineChart {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            title: "".to_string(),
            values: WindowBuffer::with_capacity(capacity),
            color: Color::White,
            point_count: 0,
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn values(&self) -> &[(f64, f64)] {
        self.values.values()
    }

    pub fn push(&mut self, value: f64) {
        let x = self.point_count as f64;
        self.point_count += 1;
        self.add_value((x, value));
    }

    pub fn add_value(&mut self, value: (f64, f64)) {
        self.values.push(value);
    }
}

impl LineChart for RollingLineChart {
    fn values(&self) -> &[(f64, f64)] {
        self.values()
    }

    fn color(&self) -> Color {
        self.color()
    }

    fn title(&self) -> &str {
        self.title()
    }
}

pub struct DistributionLineChart {
    title: String,
    values: Vec<(f64, f64)>,
    color: Color,
}

impl DistributionLineChart {
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl From<&[f32]> for DistributionLineChart {
    fn from(values: &[f32]) -> Self {
        Self::from(values.to_vec())
    }
}

impl From<Vec<f32>> for DistributionLineChart {
    fn from(values: Vec<f32>) -> Self {
        Self {
            title: "".to_string(),
            values: values
                .into_iter()
                .enumerate()
                .map(|(i, value)| (i as f64, value as f64))
                .collect(),
            color: Color::White,
        }
    }
}

impl LineChart for DistributionLineChart {
    fn values(&self) -> &[(f64, f64)] {
        &self.values
    }

    fn color(&self) -> Color {
        self.color
    }

    fn title(&self) -> &str {
        &self.title
    }
}

#[cfg(test)]
mod tests {
    use crate::chart::{ChartView, History, Scale, Transform};

    #[test]
    fn transforms_round_trip() {
        for t in [Transform::Identity, Transform::Log10, Transform::SymLog] {
            let ys: &[f64] = match t {
                Transform::Log10 => &[1e-9, 0.001, 1.0, 99.0, 1e9],
                _ => &[-1234.5, -1.0, 0.0, 0.001, 1.0, 99.0, 1e9],
            };
            for &y in ys {
                let back = t.to_value(t.to_chart(y));
                assert!(
                    (back - y).abs() <= 1e-9 * y.abs().max(1.0),
                    "{t:?}: {y} -> {back}"
                );
            }
        }
    }

    #[test]
    fn log_uses_a_true_log_unless_values_reach_zero() {
        let log = ChartView {
            scale: Scale::Log,
            history: History::All,
        };
        assert_eq!(
            log.transform([1e-5, 0.3, 40.0].into_iter()),
            Transform::Log10
        );
        assert_eq!(log.transform([0.0, 0.3].into_iter()), Transform::SymLog);
        assert_eq!(log.transform([-2.0, 0.3].into_iter()), Transform::SymLog);
        let linear = ChartView::default();
        assert_eq!(linear.transform([1.0].into_iter()), Transform::Identity);
    }

    #[test]
    fn history_keeps_the_newest_points() {
        let view = ChartView {
            scale: Scale::Linear,
            history: History::Last(2),
        };
        let values = [(0.0, 5.0), (1.0, 6.0), (2.0, 7.0)];
        assert_eq!(view.visible(&values), &[(1.0, 6.0), (2.0, 7.0)]);
    }

    #[test]
    fn history_cycles_back_to_all() {
        let mut view = ChartView::default();
        view.next_history();
        view.next_history();
        view.next_history();
        assert_eq!(view.history, History::All);
        assert_eq!(view.label(), None);
    }
}
