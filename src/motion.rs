//! Declarative element animation via [`MotionExt::motion`].
//!
//! Wraps any GPUI element with animated property transitions
//! driven by [`gpui::Transition`] under the hood.

use std::{sync::Arc, time::Duration};

use gpui::{Element, ElementId, IntoElement, Stateful, point, px, size};

use crate::Easing;

/// Target animation values and timing configuration.
///
/// Constructed via builder methods inside the [`MotionExt::motion`] closure:
///
/// ```ignore
/// .motion(|m| m.opacity(0.5).x(100.0).duration(Duration::from_millis(500)))
/// ```
#[derive(Debug, Clone)]
pub struct MotionBuilder {
    pub opacity: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub duration: Duration,
    pub easing: Easing,
    pub scale: Option<f32>,
}

impl Default for MotionBuilder {
    fn default() -> Self {
        Self {
            opacity: None,
            x: None,
            y: None,
            duration: Duration::from_millis(300),
            easing: Easing::EaseOutCubic,
            scale: None,
        }
    }
}

impl MotionBuilder {
    /// Set the target opacity (0.0–1.0).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    /// Set the target x translation in pixels.
    pub fn x(mut self, x: f32) -> Self {
        self.x = Some(x);
        self
    }

    /// Set the target y translation in pixels.
    pub fn y(mut self, y: f32) -> Self {
        self.y = Some(y);
        self
    }

    /// Set the transition duration.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the easing curve.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }
}

/// An element wrapper that drives animated property transitions.
///
/// Created via [`MotionExt::motion()`]. During each frame:
///
/// 1. **Prepaint** — evaluates all animated properties via
///    [`gpui::Transition`] and caches the current values.
/// 2. **Paint** — applies the cached opacity and translation offset
///    before painting the inner element.
pub struct Motion<E> {
    inner: E,
    config: MotionBuilder,
    current_opacity: f32,
    current_x: f32,
    current_y: f32,
    current_scale: f32,
}

/// Extension trait that adds [`.motion()`](MotionExt::motion) to elements with an ID.
///
/// Only available on elements that have called [`.id()`](gpui::InteractiveElement::id),
/// because a stable element ID is required for persistent animation state.
pub trait MotionExt: Sized {
    /// Wrap this element with an animated property transition.
    ///
    /// The closure receives a [`MotionBuilder`] and should return the
    /// desired target values via chainable builder methods.
    fn motion(self, f: impl FnOnce(MotionBuilder) -> MotionBuilder) -> Motion<Self> {
        Motion {
            inner: self,
            config: f(MotionBuilder::default()),
            current_opacity: 1.0,
            current_x: 0.0,
            current_y: 0.0,
            current_scale: 1.0,
        }
    }
}

impl<E: Element> MotionExt for Stateful<E> {}

impl<E: Element> Motion<E> {
    /// Shared transition logic for a single animatable property:
    /// create (or retrieve) a keyed transition, update the target,
    /// and evaluate the current interpolated value.
    fn animate_property<T: gpui::Lerp + Clone + PartialEq + 'static>(
        &mut self,
        key_suffix: &str,
        target: T,
        start: T,
        base_id: &ElementId,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> T {
        let key = ElementId::NamedChild(Arc::new(base_id.clone()), key_suffix.into());
        let easing = self.config.easing;
        let t = window
            .use_keyed_transition(key, cx, self.config.duration, |_, _| start.clone())
            .with_easing(move |t| crate::easing::apply(easing, t));
        t.update(cx, |val, _| *val = target);
        t.evaluate(window, cx).clone()
    }
}

impl<E: Element> Element for Motion<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        self.inner.id()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        let base_id = self.inner.id().unwrap();

        // Evaluate each animated property and cache for paint.
        if let Some(target) = self.config.opacity {
            self.current_opacity =
                self.animate_property("opacity", target, 1.0, &base_id, window, cx);
        }
        if let Some(target) = self.config.x {
            self.current_x = self.animate_property("x", target, 0.0, &base_id, window, cx);
        }
        if let Some(target) = self.config.y {
            self.current_y = self.animate_property("y", target, 0.0, &base_id, window, cx);
        }
        if let Some(target) = self.config.scale {
            self.current_scale = self.animate_property("scale", target, 1.0, &base_id, window, cx);
        }

        // Offset affects both hit testing (prepaint) and rendering (paint).
        let offset = point(px(self.current_x), px(self.current_y));
        window.with_element_offset(offset, |window| {
            self.inner
                .prepaint(id, inspector_id, bounds, request_layout, window, cx)
        })
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        // Skip the opacity wrapper when already at full opacity.
        let opacity = if (self.current_opacity - 1.0).abs() > f32::EPSILON {
            Some(self.current_opacity)
        } else {
            None
        };

        let offset = point(px(self.current_x), px(self.current_y));
        let offset_bounds = bounds + offset;

        // Scale bounds from center — equivalent to a GPU transform for uniform scale.
        let scale = self.current_scale;
        let paint_bounds = if (scale - 1.0).abs() > f32::EPSILON {
            let c = offset_bounds.center();
            let hw = offset_bounds.size.width * scale * 0.5;
            let hh = offset_bounds.size.height * scale * 0.5;
            gpui::Bounds {
                origin: point(c.x - hw, c.y - hh),
                size: size(hw * 2.0, hh * 2.0),
            }
        } else {
            offset_bounds
        };

        window.with_element_opacity(opacity, |window| {
            self.inner.paint(
                id,
                inspector_id,
                paint_bounds,
                request_layout,
                prepaint,
                window,
                cx,
            );
        });
    }
}

impl<T: gpui::Element> IntoElement for Motion<T> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn config_default_duration_is_300ms() {
        let c = MotionBuilder::default();
        assert_eq!(c.duration, Duration::from_millis(300));
    }

    #[test]
    fn config_default_easing_is_ease_out_cubic() {
        let c = MotionBuilder::default();
        assert_eq!(c.easing, Easing::EaseOutCubic);
    }

    #[test]
    fn config_default_no_properties_animated() {
        let c = MotionBuilder::default();
        assert!(c.opacity.is_none());
        assert!(c.x.is_none());
        assert!(c.y.is_none());
    }

    #[test]
    fn config_builder_sets_opacity() {
        let c = MotionBuilder::default().opacity(0.5);
        assert_eq!(c.opacity, Some(0.5));
    }

    #[test]
    fn config_builder_sets_x() {
        let c = MotionBuilder::default().x(100.0);
        assert_eq!(c.x, Some(100.0));
    }

    #[test]
    fn config_builder_sets_y() {
        let c = MotionBuilder::default().y(50.0);
        assert_eq!(c.y, Some(50.0));
    }

    #[test]
    fn config_builder_sets_duration() {
        let c = MotionBuilder::default().duration(Duration::from_millis(500));
        assert_eq!(c.duration, Duration::from_millis(500));
    }

    #[test]
    fn config_builder_sets_easing() {
        let c = MotionBuilder::default().easing(Easing::EaseInBack);
        assert_eq!(c.easing, Easing::EaseInBack);
    }

    #[test]
    fn config_builder_chains_multiple_properties() {
        let c = MotionBuilder::default()
            .opacity(0.3)
            .x(200.0)
            .y(100.0)
            .duration(Duration::from_millis(1000))
            .easing(Easing::EaseOutBounce);
        assert_eq!(c.opacity, Some(0.3));
        assert_eq!(c.x, Some(200.0));
        assert_eq!(c.y, Some(100.0));
        assert_eq!(c.duration, Duration::from_millis(1000));
        assert_eq!(c.easing, Easing::EaseOutBounce);
    }

    #[test]
    fn config_is_clone() {
        let c1 = MotionBuilder::default().opacity(0.5).x(100.0);
        let c2 = c1.clone();
        assert_eq!(c1.opacity, c2.opacity);
        assert_eq!(c1.x, c2.x);
    }
}
