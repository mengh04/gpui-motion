//! Demonstrates the declarative `.motion()` API for animated opacity and translation.

use std::time::Duration;

use gpui::{
    App, AppContext, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window, WindowBounds, WindowOptions, div, px, size,
};

use gpui_motion::{Easing, MotionExt};

struct DemoView;

impl Render for DemoView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::white())
            .child(
                div()
                    .id("animated-box")
                    .size(px(96.))
                    // .rounded_md()
                    .bg(gpui::black())
                    .motion(|m| {
                        m.opacity(0.2)
                            .x(200.)
                            .duration(Duration::from_millis(1000))
                            .scale(2.)
                            .rotate(60.)
                            .easing(Easing::EaseOutBounce)
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
