use crate::{
    state::AppState,
    styles::{TEXT_FG_COLOR, TREND_NEUTRAL_COLOR},
    widgets::{AppWidget, FnWidget, Panel, panels::search::kv},
};
use radiate_engines::Chromosome;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
};
use std::fmt::Debug;

/// Raw `Debug` dump of the phenotype selected in the population table: a short
/// summary (id, age, score, species), then one chromosome at a time, pretty-printed.
/// Tab cycles chromosomes and J/K/D/U scroll; long lines wrap rather than clip.
pub struct DebugPanel;

impl DebugPanel {
    pub fn new() -> Self {
        DebugPanel
    }
}

impl<C: Chromosome + Debug> AppWidget<C> for DebugPanel {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>) {
        let row = state.tables.population.selected_row;
        let count = state.selected_chromosome_count();
        state.inspect.sync(row, count);

        let Some(phenotype) = state.get_selected_phenotype() else {
            Panel::empty("No phenotype selected").render(area, buf);
            return;
        };

        let index = state.inspect.chromosome;

        let body = match phenotype.genotype().get(index) {
            Some(chromosome) => {
                format!("{chromosome:#?}")
            }
            None => "Empty genotype".to_string(),
        };

        let title = Line::from(Span::styled(
            format!(" Phenotype {} ", phenotype.id().as_ref()),
            Style::default().fg(Color::White).bold(),
        ));

        let tab = Line::from(Span::styled(
            " [Tab] ".to_string(),
            Style::default().fg(Color::Green).bold(),
        ))
        .right_aligned();

        let mut chrome_title = Vec::new();
        for i in 0..count {
            if i == index {
                chrome_title.push(Span::styled(
                    format!("[{}]", i + 1),
                    Style::default().fg(Color::LightGreen),
                ));
            } else {
                chrome_title.push(Span::styled(
                    format!(" {} ", i + 1),
                    Style::default().fg(TREND_NEUTRAL_COLOR),
                ));
            }
        }

        let bottom_left = Line::from(
            vec![kv("J/K", "scroll"), kv("D/U", "page")]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        );

        let inspect = &mut state.inspect;
        let content = FnWidget::new(|area: Rect, buf: &mut Buffer| {
            let [text_area, bar_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
            let rows = wrap_lines(&body, text_area.width as usize);
            let total = rows.len();
            let page = text_area.height as usize;
            inspect.set_content_rows(total, page);

            let visible = rows
                .into_iter()
                .skip(inspect.scroll)
                .take(page)
                .map(|r| Line::styled(r, Style::default().fg(TEXT_FG_COLOR)))
                .collect::<Vec<_>>();
            Paragraph::new(visible).render(text_area, buf);

            if total > page {
                let mut bar =
                    ScrollbarState::new(total.saturating_sub(page)).position(inspect.scroll);
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .track_style(Style::default().fg(Color::DarkGray))
                    .thumb_style(Style::default().fg(Color::LightGreen))
                    .render(bar_area, buf, &mut bar);
            }
        });

        Panel::new(content)
            .title_top_right(tab)
            .title_top_left(title)
            .titled_bottom(chrome_title)
            .title_bottom_left(bottom_left)
            .render(area, buf);
    }
}

/// Hard-wraps each line to `width` columns. A continuation row keeps the line's
/// indent plus two, so wrapped `{:#?}` fields still read as nested. Debug output is
/// close enough to ASCII that a char stands in for a column.
fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    if width == 0 {
        return rows;
    }

    for line in text.lines() {
        let chars = line.chars().collect::<Vec<_>>();
        if chars.len() <= width {
            rows.push(line.to_string());
            continue;
        }

        let indent = chars.iter().take_while(|c| c.is_whitespace()).count() + 2;
        let indent = if indent < width / 2 { indent } else { 0 };

        let (first, mut rest) = chars.split_at(width);
        rows.push(first.iter().collect());
        while !rest.is_empty() {
            let (chunk, tail) = rest.split_at(rest.len().min(width - indent));
            rows.push(" ".repeat(indent) + &chunk.iter().collect::<String>());
            rest = tail;
        }
    }

    rows
}
