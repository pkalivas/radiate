use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Clear, StatefulWidget, Widget},
};

pub struct ModalWidget<W> {
    title: Option<String>,
    width_pct: u16,
    height_pct: u16,
    block_style: Style,
    overlay_style: Style,
    child: W,
}

impl<W> ModalWidget<W> {
    pub fn new(child: W) -> Self {
        Self {
            title: None,
            width_pct: 70,
            height_pct: 80,
            block_style: Style::default().bg(crate::styles::ALT_BG_COLOR),
            overlay_style: Style::default().bg(crate::styles::OVERLAY_COLOR),
            child,
        }
    }
}

impl<W> ModalWidget<W> {
    fn centered_rect(r: Rect, height: u16, width: u16) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - height) / 2),
                Constraint::Percentage(height),
                Constraint::Percentage((100 - height) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - width) / 2),
                Constraint::Percentage(width),
                Constraint::Percentage((100 - width) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl<W> Widget for ModalWidget<W>
where
    W: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = Self::centered_rect(area, self.height_pct, self.width_pct);

        Block::default().style(self.overlay_style).render(area, buf);

        Clear.render(popup_area, buf);

        let mut block = Block::default().style(self.block_style);
        if let Some(title) = self.title {
            block = block.title(title);
        }

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);
        self.child.render(inner, buf);
    }
}

impl<W> StatefulWidget for ModalWidget<W>
where
    W: StatefulWidget,
{
    type State = W::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let popup_area = Self::centered_rect(area, self.height_pct, self.width_pct);

        Block::default().style(self.overlay_style).render(area, buf);

        Clear.render(popup_area, buf);

        let mut block = Block::default().style(self.block_style);
        if let Some(title) = self.title {
            block = block.title(title);
        }

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);
        self.child.render(inner, buf, state);
    }
}
