use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Clear, Widget},
};

pub struct ModalWidget<W> {
    title: Option<String>,
    width_pct: u16,
    height_pct: u16,
    block_style: Style,
    child: W,
}

impl<W> ModalWidget<W>
where
    W: Widget,
{
    pub fn new(child: W) -> Self {
        Self {
            title: None,
            width_pct: 70,
            height_pct: 80,
            block_style: Style::default(),
            child,
        }
    }

    fn centered_rect(&self, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - self.height_pct) / 2),
                Constraint::Percentage(self.height_pct),
                Constraint::Percentage((100 - self.height_pct) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - self.width_pct) / 2),
                Constraint::Percentage(self.width_pct),
                Constraint::Percentage((100 - self.width_pct) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl<W> Widget for ModalWidget<W>
where
    W: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = self.centered_rect(area);

        Clear.render(area, buf);

        let mut block = Block::default().style(self.block_style);
        if let Some(title) = self.title {
            block = block.title(title);
        }

        let inner = block.inner(area);
        block.render(area, buf);

        self.child.render(inner, buf);
    }
}
