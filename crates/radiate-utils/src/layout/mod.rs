mod error;
mod index_impls;
mod indices;
mod layout;
mod shape;
mod view;

pub use error::LayoutError;
pub use indices::Indices;
pub use layout::Layout;
pub use shape::{Shape, Strides};

pub use view::{LayoutOwnedView, LayoutOwnedViewMut, LayoutView, LayoutViewMut};
