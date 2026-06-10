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
    top_right_title: Option<Line<'static>>,
    block: Block<'static>,
    render_inside_block: bool,
}

impl<W: Widget> Panel<W> {
    pub fn new(child: W) -> Self {
        Self {
            title: None,
            child: Some(child),
            title_bottom: None,
            top_right_title: None,
            render_inside_block: true,
            block: Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        }
    }

    pub fn titled(mut self, title: impl Into<Line<'static>>) -> Self {
        self.title = Some(title.into());
        self
    }

    // pub fn focused(mut self, focused: bool) -> Self {
    //     if focused {
    //         self.block = self
    //             .block
    //             .border_style(ratatui::style::Style::default().fg(crate::styles::BORDER_GREEN));
    //     }
    //     self
    // }

    // pub fn bordered(mut self, block: Block<'static>) -> Self {
    //     self.block = block;
    //     self
    // }

    // pub fn titled_bottom(mut self, title: impl Into<Line<'static>>) -> Self {
    //     self.title_bottom = Some(title.into());
    //     self
    // }

    // pub fn title_top_right(mut self, title: impl Into<Line<'static>>) -> Self {
    //     let line = title.into();
    //     self.top_right_title = Some(line);
    //     self
    // }

    #[allow(dead_code)]
    pub fn render_inside_block(mut self, render_inside: bool) -> Self {
        self.render_inside_block = render_inside;
        self
    }
}

impl Panel<Empty> {
    pub fn empty(txt: &str) -> Self {
        Self {
            child: Some(Empty::new(txt)),
            title: None,
            title_bottom: None,
            top_right_title: None,
            render_inside_block: true,
            block: Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
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
            top_right_title: None,
            render_inside_block: true,
            block: Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
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
        if self.title.is_none() && self.title_bottom.is_none() && !self.render_inside_block {
            if let Some(child) = self.child {
                child.render(area, buf);
            }
        } else if self.title.is_some() || self.title_bottom.is_some() || self.render_inside_block {
            let mut block = self.block;

            if let Some(title) = self.title {
                block = block.title(title).title_alignment(Alignment::Center);
            }

            if let Some(title_bottom) = self.title_bottom {
                block = block
                    .title_bottom(title_bottom)
                    .title_alignment(Alignment::Center);
            }

            if let Some(top_right_title) = self.top_right_title {
                block = block.title(top_right_title.right_aligned())
            }

            if self.render_inside_block {
                let inner = block.inner(area);
                block.render(area, buf);

                if let Some(child) = self.child {
                    child.render(inner, buf);
                }
            } else {
                block.render(area, buf);

                if let Some(child) = self.child {
                    child.render(area, buf);
                }
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
