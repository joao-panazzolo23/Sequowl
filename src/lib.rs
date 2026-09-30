//! Sequowl's application library.
//!
//! The editor core (`core`) and the Slint presentation layer (`presentation`)
//! live in the library so they stay testable and reusable, while `main.rs`
//! only wires them to the window and the demo state.

slint::include_modules!();

pub mod core;
pub mod presentation;
