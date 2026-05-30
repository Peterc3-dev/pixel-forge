//! pixel-forge core: canvas model, drawing primitives, palette, and color types.
//!
//! The binary ([`main.rs`](../src/main.rs)) drives a ratatui TUI on top of this
//! library. Keeping the pure logic here makes it unit-testable independently of
//! terminal I/O.

pub mod canvas;
pub mod color;
