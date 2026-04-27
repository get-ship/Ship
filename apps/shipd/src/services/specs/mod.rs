//! Specs service — view-shaped read ops for the Specs view.
//!
//! Pure data shaping. Body-parsing helpers in `parse`; rail / detail
//! builders in `view_state`. Registry dispatches to these.

pub mod parse;
pub mod service;
pub mod view_state;
