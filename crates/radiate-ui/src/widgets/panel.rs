use radiate_utils::intern;
use ratatui::text::Line;
use ratatui::widgets::BorderType;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Widget},
};

pub struct Panel<W: Widget> {
    child: Option<W>,
    title: Option<Line<'static>>,
    title_bottom: Option<Line<'static>>,
}

impl<W: Widget> Panel<W> {
    pub fn new(child: W) -> Self {
        Self {
            title: None,
            child: Some(child),
            title_bottom: None,
        }
    }

    pub fn titled(mut self, title: impl Into<Line<'static>>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn titled_bottom(mut self, title: impl Into<Line<'static>>) -> Self {
        self.title_bottom = Some(title.into());
        self
    }
}

impl Panel<Empty> {
    pub fn empty(txt: &str) -> Self {
        Self {
            child: Some(Empty::new(txt)),
            title: None,
            title_bottom: None,
        }
    }
}

impl<W> Default for Panel<W>
where
    W: Widget,
{
    fn default() -> Self {
        Self {
            child: None,
            title: None,
            title_bottom: None,
        }
    }
}

impl<'a, F> From<F> for Panel<FnWidget<'a, F>>
where
    F: 'a + FnOnce(Rect, &mut Buffer),
{
    fn from(value: F) -> Self {
        Panel::new(FnWidget::new(value))
    }
}

impl<W> Widget for Panel<W>
where
    W: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.title.is_none() && self.title_bottom.is_none() {
            if let Some(child) = self.child {
                child.render(area, buf);
            }
            return;
        } else if self.title.is_some() || self.title_bottom.is_some() {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);

            if let Some(title) = self.title {
                block = block.title(title).title_alignment(Alignment::Center);
            }

            if let Some(title_bottom) = self.title_bottom {
                block = block
                    .title_bottom(title_bottom)
                    .title_alignment(Alignment::Center);
            }

            let inner = block.inner(area);
            block.render(area, buf);

            if let Some(child) = self.child {
                child.render(inner, buf);
            }
        }
    }
}

pub struct FnWidget<'a, F>
where
    F: 'a + FnOnce(Rect, &mut Buffer),
{
    f: Option<F>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, F> FnWidget<'a, F>
where
    F: 'a + FnOnce(Rect, &mut Buffer),
{
    pub fn new(f: F) -> Self {
        Self {
            f: Some(f),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, F> Widget for FnWidget<'a, F>
where
    F: FnOnce(Rect, &mut Buffer),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(f) = self.f {
            f(area, buf);
        }
    }
}

impl<'a, F> From<F> for FnWidget<'a, F>
where
    F: 'a + FnOnce(Rect, &mut Buffer),
{
    fn from(value: F) -> Self {
        Self::new(value)
    }
}

pub struct Empty {
    msg: &'static str,
}

impl Empty {
    pub fn new(msg: &str) -> Self {
        Self { msg: intern!(msg) }
    }
}

impl Widget for Empty {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(format!(" {} ", self.msg)).centered())
            .render(area, buf);
    }
}
