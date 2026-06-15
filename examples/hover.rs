//! Hover fade — minimal test of the gpui::Transition cache fix.
//!
//! 用 gpui 官方 `Transition<T>` + `use_keyed_transition`（无 spring，无 cubic，
//! 用 gpui 自带 ease_in_out）。目的是验证 patch 后动画能跑起来。

use std::time::Duration;

use gpui::prelude::StatefulInteractiveElement;
use gpui::{
    App, AppContext, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window, WindowBounds, WindowOptions, div, ease_in_out, px, size,
};

struct HoverView;

impl Render for HoverView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let opacity = window
            .use_keyed_transition(
                "hover-fade-opacity",
                cx,
                Duration::from_millis(300),
                |_, _| 1.0_f32,
            )
            .with_easing(ease_in_out);

        // evaluate 当前返回 Ref<T>（patch 没在），用 * 解引用
        let current = *opacity.evaluate(window, cx);

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::white())
            .child(
                div()
                    .id("hover-box") // 必须 .id() 才能挂事件
                    .size(px(96.))
                    .rounded_md()
                    .bg(gpui::black())
                    .opacity(current) // ← 这就是动画值
                    .on_hover(move |&hovered, _window, cx| {
                        // update 闭包里的 cx 是 Context<TransitionState<T>>，
                        // 调 cx.notify() 通知 transition state 脏，触发 view re-render
                        opacity.update(cx, |v, cx| {
                            *v = if hovered { 0.2 } else { 1.0 };
                            cx.notify();
                        });
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
            |_window, cx| cx.new(|_| HoverView),
        )
        .unwrap();
    });
}
