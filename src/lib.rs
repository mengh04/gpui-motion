//! Declarative animation primitives for the [GPUI](https://www.gpui.rs/) UI framework.
//!
//! # Quick start
//!
//! ```ignore
//! use gpui_motion::MotionExt;
//!
//! div()
//!     .id("my-element")
//!     .size(px(96.))
//!     .bg(gpui::black())
//!     .motion(|m| m.opacity(0.2).x(100.0))
//! ```

pub mod easing;
pub mod lerp;
pub mod motion;
pub mod tween;

pub use easing::Easing;
pub use motion::{MotionBuilder, MotionExt};
pub use tween::Tween;
