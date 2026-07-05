use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Tabs, Widget},
};

#[derive(Default)]
pub struct TabComponent<'a> {
    options: Vec<Line<'a>>,
    selected: usize,
}

impl<'a> TabComponent<'a> {
    pub fn new<Iter>(options: Iter) -> Self
    where
        Iter: IntoIterator,
        Iter::Item: Into<Line<'a>>,
    {
        let options = options.into_iter().map(Into::into).collect();

        Self {
            options,
            selected: 0,
        }
    }

    pub fn select(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected = index;
        }
        self
    }
}

impl<'a> From<Vec<Line<'a>>> for TabComponent<'a> {
    fn from(value: Vec<Line<'a>>) -> Self {
        Self::new(value)
    }
}

impl<'a> From<Vec<&'a str>> for TabComponent<'a> {
    fn from(value: Vec<&'a str>) -> Self {
        Self::from(&value)
    }
}

impl<'a> From<&Vec<&'a str>> for TabComponent<'a> {
    fn from(value: &Vec<&'a str>) -> Self {
        Self::new(
            value
                .iter()
                .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White))),
        )
    }
}

impl<'a> Widget for TabComponent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Tabs::new(
            self.options
                .iter()
                .map(|t| Span::styled(format!(" {t} "), Style::default().fg(Color::White))),
        )
        .select(self.selected)
        .highlight_style(crate::styles::selected_item_style())
        .bold()
        .render(area, buf)
    }
}
