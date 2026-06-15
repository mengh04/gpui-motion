//! Declarative element animation via [`MotionExt::motion`].
//!
//! Wraps any GPUI element with animated property transitions
//! driven by [`gpui::Transition`] under the hood.

use std::{sync::Arc, time::Duration};

use gpui::{Element, ElementId, IntoElement, Stateful, TransformationMatrix, point, px, size};

use crate::Easing;

/// Per-property animation target values.
///
/// Used inside [`MotionBuilder::initial`] and [`MotionBuilder::animate`] closures.
#[derive(Debug, Clone, Default)]
pub struct PropertyTarget {
    pub opacity: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub scale: Option<f32>,
    pub rotate: Option<f32>,
}

impl PropertyTarget {
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

    /// Set the target uniform scale (1.0 = no scaling).
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Set the target rotation in degrees.
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotate = Some(degrees * std::f32::consts::PI / 180.0);
        self
    }

    /// Set the target rotation in radians.
    pub fn rotate_radians(mut self, radians: f32) -> Self {
        self.rotate = Some(radians);
        self
    }
}

/// Animation configuration for a motion element.
///
/// Constructed via builder methods inside the [`MotionExt::motion`] closure.
/// The shorthand methods (`opacity`, `x`, etc.) set [`animate`](Self::animate);
/// use [`initial`](Self::initial) + [`animate`](Self::animate) for two-state
/// entrance animations.
///
/// ```ignore
/// // Shorthand — animate from current value to target
/// .motion(|m| m.opacity(0.5).x(100.0))
///
/// // Two-state — animate from initial to target
/// .motion(|m| m
///     .initial(|s| s.opacity(0.0).y(-20.0))
///     .animate(|s| s.opacity(1.0).y(0.0))
/// )
/// ```
#[derive(Debug, Clone)]
pub struct MotionBuilder {
    /// Optional initial state (applied immediately on first render).
    pub initial: Option<PropertyTarget>,
    /// Target state to animate toward.
    pub animate: Option<PropertyTarget>,
    /// Transition duration.
    pub duration: Duration,
    /// Easing curve for all animated properties.
    pub easing: Easing,
}

impl Default for MotionBuilder {
    fn default() -> Self {
        Self {
            initial: None,
            animate: None,
            duration: Duration::from_millis(300),
            easing: Easing::EaseOutCubic,
        }
    }
}

impl MotionBuilder {
    /// Set the initial state. The element appears in this state on first
    /// render, then transitions toward [`animate`](Self::animate).
    pub fn initial(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
        self.initial = Some(f(PropertyTarget::default()));
        self
    }

    /// Set the target state to animate toward.
    pub fn animate(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
        self.animate = Some(f(PropertyTarget::default()));
        self
    }

    // ── Shorthand methods (merge into existing animate) ──

    /// Shorthand for `animate(|s| s.opacity(opacity))`.
    pub fn opacity(mut self, opacity: f32) -> Self {
        let existing = self.animate.take().unwrap_or_default();
        self.animate = Some(existing.opacity(opacity));
        self
    }

    /// Shorthand for `animate(|s| s.x(x))`.
    pub fn x(mut self, x: f32) -> Self {
        let existing = self.animate.take().unwrap_or_default();
        self.animate = Some(existing.x(x));
        self
    }

    /// Shorthand for `animate(|s| s.y(y))`.
    pub fn y(mut self, y: f32) -> Self {
        let existing = self.animate.take().unwrap_or_default();
        self.animate = Some(existing.y(y));
        self
    }

    /// Shorthand for `animate(|s| s.scale(scale))`.
    pub fn scale(mut self, scale: f32) -> Self {
        let existing = self.animate.take().unwrap_or_default();
        self.animate = Some(existing.scale(scale));
        self
    }

    /// Shorthand for `animate(|s| s.rotate(degrees))`.
    pub fn rotate(mut self, degrees: f32) -> Self {
        let existing = self.animate.take().unwrap_or_default();
        self.animate = Some(existing.rotate(degrees));
        self
    }

    /// Shorthand for `animate(|s| s.rotate_radians(radians))`.
    pub fn rotate_radians(mut self, radians: f32) -> Self {
        let existing = self.animate.take().unwrap_or_default();
        self.animate = Some(existing.rotate_radians(radians));
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
}

/// An element wrapper that drives animated property transitions.
///
/// Created via [`MotionExt::motion()`]. During each frame:
///
/// 1. **Prepaint** — evaluates all animated properties via
///    [`gpui::Transition`] and caches the current values.
/// 2. **Paint** — applies the cached opacity, translation offset,
///    scale, and rotation before painting the inner element.
pub struct Motion<E> {
    inner: E,
    config: MotionBuilder,
    current_opacity: f32,
    current_x: f32,
    current_y: f32,
    current_scale: f32,
    current_rotate: f32,
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
        let config = f(MotionBuilder::default());

        // Start from the initial state if one was provided, otherwise
        // use the neutral defaults.
        let init_opacity = config
            .initial
            .as_ref()
            .and_then(|i| i.opacity)
            .unwrap_or(1.0);
        let init_x = config.initial.as_ref().and_then(|i| i.x).unwrap_or(0.0);
        let init_y = config.initial.as_ref().and_then(|i| i.y).unwrap_or(0.0);
        let init_scale = config.initial.as_ref().and_then(|i| i.scale).unwrap_or(1.0);
        let init_rotate = config
            .initial
            .as_ref()
            .and_then(|i| i.rotate)
            .unwrap_or(0.0);

        Motion {
            inner: self,
            config,
            current_opacity: init_opacity,
            current_x: init_x,
            current_y: init_y,
            current_scale: init_scale,
            current_rotate: init_rotate,
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
        if let Some(target) = self.config.animate.as_ref().and_then(|a| a.opacity) {
            self.current_opacity = self.animate_property(
                "opacity",
                target,
                self.current_opacity,
                &base_id,
                window,
                cx,
            );
        }
        if let Some(target) = self.config.animate.as_ref().and_then(|a| a.x) {
            self.current_x =
                self.animate_property("x", target, self.current_x, &base_id, window, cx);
        }
        if let Some(target) = self.config.animate.as_ref().and_then(|a| a.y) {
            self.current_y =
                self.animate_property("y", target, self.current_y, &base_id, window, cx);
        }
        if let Some(target) = self.config.animate.as_ref().and_then(|a| a.scale) {
            self.current_scale =
                self.animate_property("scale", target, self.current_scale, &base_id, window, cx);
        }
        if let Some(target) = self.config.animate.as_ref().and_then(|a| a.rotate) {
            self.current_rotate =
                self.animate_property("rotate", target, self.current_rotate, &base_id, window, cx);
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
        let paint_bounds = bounds + offset;

        // Build a single transform that combines scale and rotation.
        // Both are applied around the element center by paint_quad.
        let scale = self.current_scale;
        let rotate = self.current_rotate;
        let needs_xform = (scale - 1.0).abs() > f32::EPSILON || rotate.abs() > f32::EPSILON;

        window.with_element_opacity(opacity, |window| {
            if needs_xform {
                let xform = TransformationMatrix::unit()
                    .scale(size(scale, scale))
                    .rotate(gpui::radians(rotate));
                window.with_element_transform(xform, |window| {
                    self.inner.paint(
                        id,
                        inspector_id,
                        paint_bounds,
                        request_layout,
                        prepaint,
                        window,
                        cx,
                    );
                })
            } else {
                self.inner.paint(
                    id,
                    inspector_id,
                    paint_bounds,
                    request_layout,
                    prepaint,
                    window,
                    cx,
                );
            }
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
    fn config_default_no_animate_target() {
        let c = MotionBuilder::default();
        assert!(c.animate.is_none());
        assert!(c.initial.is_none());
    }

    #[test]
    fn shorthand_opacity_sets_animate() {
        let c = MotionBuilder::default().opacity(0.5);
        assert_eq!(c.animate.unwrap().opacity, Some(0.5));
    }

    #[test]
    fn shorthand_x_sets_animate() {
        let c = MotionBuilder::default().x(100.0);
        assert_eq!(c.animate.unwrap().x, Some(100.0));
    }

    #[test]
    fn shorthand_y_sets_animate() {
        let c = MotionBuilder::default().y(50.0);
        assert_eq!(c.animate.unwrap().y, Some(50.0));
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
    fn config_builder_chains_shorthand_properties() {
        let c = MotionBuilder::default()
            .opacity(0.3)
            .x(200.0)
            .y(100.0)
            .duration(Duration::from_millis(1000))
            .easing(Easing::EaseOutBounce);
        let a = c.animate.unwrap();
        assert_eq!(a.opacity, Some(0.3));
        assert_eq!(a.x, Some(200.0));
        assert_eq!(a.y, Some(100.0));
        assert_eq!(c.duration, Duration::from_millis(1000));
        assert_eq!(c.easing, Easing::EaseOutBounce);
    }

    #[test]
    fn initial_and_animate_two_state() {
        let c = MotionBuilder::default()
            .initial(|s| s.opacity(0.0).y(-20.0))
            .animate(|s| s.opacity(1.0).y(0.0))
            .duration(Duration::from_millis(500));
        let i = c.initial.unwrap();
        let a = c.animate.unwrap();
        assert_eq!(i.opacity, Some(0.0));
        assert_eq!(i.y, Some(-20.0));
        assert_eq!(a.opacity, Some(1.0));
        assert_eq!(a.y, Some(0.0));
        assert_eq!(c.duration, Duration::from_millis(500));
    }

    #[test]
    fn builder_is_clone() {
        let c1 = MotionBuilder::default().animate(|s| s.opacity(0.5).x(100.0));
        let c2 = c1.clone();
        let a1 = c1.animate.unwrap();
        let a2 = c2.animate.unwrap();
        assert_eq!(a1.opacity, a2.opacity);
        assert_eq!(a1.x, a2.x);
    }

    #[test]
    fn property_target_default_has_no_properties() {
        let p = PropertyTarget::default();
        assert!(p.opacity.is_none());
        assert!(p.x.is_none());
        assert!(p.y.is_none());
        assert!(p.scale.is_none());
        assert!(p.rotate.is_none());
    }
}
