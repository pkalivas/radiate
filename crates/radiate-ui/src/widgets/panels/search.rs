use crate::state::{AppState, UiMode};
use radiate_engines::Chromosome;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Borders, Paragraph, Widget},
};

pub struct SearchBarWidget<'a, C: Chromosome> {
    pub state: &'a AppState<C>,
}

impl<'a, C: Chromosome> SearchBarWidget<'a, C> {
    pub fn new(state: &'a AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for SearchBarWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = if self.state.nav.search.active {
            " Search (active) "
        } else {
            " Search (/) "
        };

        let border_style = crate::styles::panel_block(self.state.nav.is_search_focused());

        let total_renders = self.state.run.render_count;

        let renders = vec![
            " Renders: ".fg(Color::Gray).bold(),
            format!("{} ", total_renders).fg(Color::LightGreen),
        ];

        Paragraph::new(self.state.nav.search.query.as_str())
            .block(
                border_style
                    .title(title)
                    .title_bottom(help_text(self.state))
                    .title_bottom(Line::from(renders).right_aligned().fg(Color::LightBlue))
                    .style(Style::default())
                    .borders(Borders::ALL),
            )
            .style(Style::default())
            .render(area, buf);
    }
}

/// Context-sensitive footer hint: shows only the keys live in the current
/// [`UiMode`], so the control vocabulary the user has to scan stays small no
/// matter how much data the dashboard grows to hold. The full key map lives
/// behind the `?` help overlay.
pub fn help_text<C: Chromosome>(state: &AppState<C>) -> Line<'static> {
    let nav = &state.nav;
    let pause = if state.run.paused { "resume" } else { "pause" };

    let mut chips = match nav.mode {
        UiMode::Dashboard => {
            let mut v = vec![kv("j/k", "navigate"), kv("h/l", "tabs")];

            if nav.dashboard_tab.supports_metric_modal() {
                v.push(kv("↵", "expand"));
            }

            v.push(kv("/", "find"));
            v.push(kv("p", pause));
            v.push(kv("?", "help"));
            v.push(kv("q", "quit"));
            v
        }
        UiMode::MetricModal => vec![
            kv("h/l", "chart"),
            kv("↵/esc", "close"),
            kv("p", pause),
            kv("?", "help"),
        ],
        UiMode::Search => vec![kv("type", "filter"), kv("↵", "apply"), kv("esc", "cancel")],
        UiMode::Help => vec![kv("?/esc", "close")],
    }
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    chips.insert(0, Span::raw("  "));
    Line::from(chips).centered()
}

/// One `[key] description` footer chip.
fn kv(key: &str, desc: &str) -> [Span<'static>; 2] {
    [
        Span::from(format!("[{key}]")).fg(Color::LightGreen).bold(),
        Span::from(format!(" {desc}  ")).fg(Color::Gray),
    ]
}
