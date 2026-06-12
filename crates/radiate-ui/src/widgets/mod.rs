mod components;
mod modal;
mod panel;
mod panels;

pub use components::*;
pub use modal::ModalWidget;
pub use panel::{FnWidget, Panel};
pub use panels::{
    EngineStatusPanelWidget, HelpPanelWidget, MetricDetailPanelWidget, MetricModalWidget,
    MetricTableWidget, SearchBarWidget,
};

mod template;
pub use template::*;

use crate::state::AppState;
use radiate_engines::Chromosome;
use ratatui::{buffer::Buffer, layout::Rect};

/// A widget that renders against the application's [`AppState`].
///
/// This mirrors ratatui's `StatefulWidget`, but takes the state as a *trait
/// parameter* (`AppWidget<C>`) rather than an associated type. ratatui's
/// `type State = AppState<C>` leaves `C` unconstrained by the impl header
/// (error E0207), which forces every widget to carry a `PhantomData<C>` just to
/// pin the parameter. Parameterizing the trait on `C` instead lets the widget
/// structs stay free of `C` entirely.
///
/// Renders by shared reference: these widgets are stateless view logic over
/// `AppState`, so `&self` also keeps the trait object-safe (`Box<dyn AppWidget<C>>`).
pub trait AppWidget<C: Chromosome> {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<C>);
}
