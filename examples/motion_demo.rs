//! Demonstrates `.motion()` with animate, while_hover, and while_tap.
//!
//! Left box: basic animate (opacity + y + rotate).
//! Right box: interactive (hover → scale up, tap → scale down + darker).

use std::time::Duration;

use gpui::{
    App, AppContext, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window, WindowBounds, WindowOptions, div, px, size,
};

use gpui_motion::MotionExt;

struct DemoView;

impl Render for DemoView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(gpui::white())
            // ── Left: pure animate (no interaction) ──
            .child(
                div()
                    .id("animate-box")
                    .size(px(96.))
                    .rounded_md()
                    .bg(gpui::black())
                    .motion(|m| {
                        m.animate(|s| s.opacity(1.0).y(300.0).rotate(360.))
                            .while_hover(|s| s.scale(1.15).duration(Duration::from_millis(200)))
                            .while_tap(|s| {
                                s.scale(0.9)
                                    .opacity(0.7)
                                    .duration(Duration::from_millis(200))
                            })
                            .duration(Duration::from_millis(5000))
                    }),
            )
            // ── Right: hover + tap interaction ──
            .child(
                div()
                    .id("interactive-box")
                    .size(px(96.))
                    .rounded_md()
                    .bg(gpui::rgb(0x0055ff))
                    .motion(|m| {
                        m.animate(|s| s.opacity(1.0).scale(1.0))
                            .while_hover(|s| s.scale(1.15))
                            .while_tap(|s| s.scale(0.9).opacity(0.7))
                            .duration(Duration::from_millis(200))
                    }),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        let bounds = WindowBounds::centered(size(px(640.), px(480.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(bounds),
                ..Default::default()
            },
            |_window, cx| cx.new(|_| DemoView),
        )
        .unwrap();
    });
}
