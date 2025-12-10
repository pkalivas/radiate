use crate::state::AppState;
use crate::styles::{ALT_ROW_BG_COLOR, NORMAL_ROW_BG};
use radiate_engines::Chromosome;
use radiate_engines::stats::fmt_duration;
use ratatui::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Block, BorderType, Cell, Paragraph, Row, Table};

pub struct EngineBaseWidget<'a, C: Chromosome> {
    state: &'a AppState<C>,
}

impl<'a, C: Chromosome> EngineBaseWidget<'a, C> {
    pub fn new(state: &'a AppState<C>) -> Self {
        Self { state }
    }
}

impl<'a, C: Chromosome> Widget for EngineBaseWidget<'a, C> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let metrics = self.state.metrics();
        let elapsed = metrics
            .time()
            .and_then(|m| m.time_sum())
            .map(fmt_duration)
            .unwrap_or_else(|| "00:00:00.000".to_string());
        let diversity = metrics
            .diversity_ratio()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let carryover = metrics
            .carryover_rate()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let unique_members = metrics
            .unique_members()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let unique_scores = metrics
            .unique_scores()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let improvements = metrics.improvements().map(|m| m.count()).unwrap_or(0);
        let survivor_count = metrics
            .survivor_count()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);
        let new_children = metrics
            .new_children()
            .and_then(|m| m.value_mean())
            .unwrap_or(0.0);

        // multi-objective metrics
        // let front_size = metrics
        //     .front_size()
        //     .and_then(|m| m.value_mean())
        //     .unwrap_or(0.0);

        let rows = vec![
            Row::new(vec![
                Cell::from("Improvements").bold(),
                Cell::from(improvements.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Diversity").bold(),
                Cell::from(format!("{:.2}%", diversity * 100.0)),
            ]),
            Row::new(vec![
                Cell::from("Carryover").bold(),
                Cell::from(format!("{:.2}%", carryover * 100.0)),
            ]),
            Row::new(vec![
                Cell::from("Unique Members").bold(),
                Cell::from(format!("{:.2}", unique_members)),
            ]),
            Row::new(vec![
                Cell::from("Unique Scores").bold(),
                Cell::from(format!("{:.2}", unique_scores)),
            ]),
            Row::new(vec![
                Cell::from("Phenotypes").bold(),
                Cell::from(format!("{:.2}", survivor_count)),
            ]),
            Row::new(vec![
                Cell::from("Children / Gen.").bold(),
                Cell::from(format!("{:.2}", new_children)),
            ]),
        ];

        let engine_state = if self.state.is_engine_running() {
            " Running ".fg(Color::LightGreen).bold()
        } else {
            " Complete ".fg(Color::Red).bold()
        };

        let mut title = vec![
            "Gen ".fg(Color::Gray).bold(),
            format!("{}", self.state.index()).fg(Color::LightGreen),
        ];

        if self.state.objective_state.objective.is_single() {
            title.push(" | Score ".fg(Color::Gray).bold());
            title.push(format!("{:.4} ", self.state.score().as_f32()).fg(Color::LightGreen));
        } else {
            title.push(" | MOGA ".fg(Color::Gray).bold());
        }

        title.push("| Time ".fg(Color::Gray).bold());
        title.push(elapsed.clone().fg(Color::LightGreen));

        let engine_table = Table::default()
            .rows(striped_rows(rows))
            .widths(&[Constraint::Fill(1), Constraint::Fill(1)]);

        let block = Block::bordered()
            .title_top(engine_state)
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center);
        let inner = block.inner(area);
        block.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(15), Constraint::Fill(1)])
            .split(inner);

        Paragraph::new(Line::from(title).centered()).render(layout[0], buf);
        Widget::render(engine_table, layout[1], buf);
    }
}

fn striped_rows<'a>(rows: impl IntoIterator<Item = Row<'a>>) -> impl Iterator<Item = Row<'a>> {
    rows.into_iter().enumerate().map(|(i, row)| {
        let bg = if i % 2 == 0 {
            NORMAL_ROW_BG
        } else {
            ALT_ROW_BG_COLOR
        };
        row.style(Style::default().bg(bg))
    })
}
