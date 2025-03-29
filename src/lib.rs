//! Output coloring and text wrapping.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

mod stream;
pub mod stream_info;
mod style;
mod text;

pub use stream::*;
pub use style::*;
pub use text::*;
