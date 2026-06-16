//! Declarative element animation via [`MotionExt::motion`].
//!
//! Wraps any GPUI element with animated property transitions
//! driven by [`gpui::Transition`] under the hood.

use std::{cell::Cell, collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use gpui::{
    DispatchPhase, Element, ElementId, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Stateful, TransformationMatrix, point, px, size,
};

use crate::Easing;

// Interaction state persistence.
//
// Hover and tap state must survive frame boundaries. We store per-element
// Rc<Cell<bool>> in a thread-local map so event listener closures can
// write to them, while the next frame's prepaint reads the updated values.

struct InteractionCells {
    hovering: Rc<Cell<bool>>,
    tapping: Rc<Cell<bool>>,
}

thread_local! {
    static INTERACTION_STATES: std::cell::RefCell<HashMap<String, InteractionCells>> =
        std::cell::RefCell::new(HashMap::new());
}

fn state_key(base_id: &ElementId) -> String {
    format!("{base_id:?}")
}

fn get_or_init_cells(base_id: &ElementId) -> InteractionCells {
    INTERACTION_STATES.with_borrow_mut(|map| {
        let key = state_key(base_id);
        if let Some(cells) = map.get(&key) {
            InteractionCells {
                hovering: cells.hovering.clone(),
                tapping: cells.tapping.clone(),
            }
        } else {
            let cells = InteractionCells {
                hovering: Rc::new(Cell::new(false)),
                tapping: Rc::new(Cell::new(false)),
            };
            map.insert(key, InteractionCells {
                hovering: cells.hovering.clone(),
                tapping: cells.tapping.clone(),
            });
            cells
        }
    })
}

fn read_hovering(base_id: &ElementId) -> bool {
    INTERACTION_STATES.with_borrow(|map| {
        map.get(&state_key(base_id))
            .map(|c| c.hovering.get())
            .unwrap_or(false)
    })
}

fn read_tapping(base_id: &ElementId) -> bool {
    INTERACTION_STATES.with_borrow(|map| {
        map.get(&state_key(base_id))
            .map(|c| c.tapping.get())
            .unwrap_or(false)
    })
}

/// Resolve a single property value and its duration from the active
/// targets in priority order (tap → hover → animate).
///
/// If no active target provides the property, falls back to `neutral`
/// (matching Framer Motion's `fallbackAnimation`) and uses the duration
/// from the first interaction target that HAS this property — so when
/// hover/tap ends, the property transitions back at the same speed.
#[allow(clippy::too_many_arguments)]
fn resolve_property(
    tapping: bool,
    while_tap: Option<&PropertyTarget>,
    hovering: bool,
    while_hover: Option<&PropertyTarget>,
    animate: Option<&PropertyTarget>,
    get: impl Fn(&PropertyTarget) -> Option<f32>,
    neutral: f32,
    global_duration: Duration,
) -> (f32, Duration) {
    let mut fallback_dur = global_duration;

    // while_tap: active → use its value; inactive → remember its duration
    if let Some(t) = while_tap
        && let Some(v) = get(t)
    {
        fallback_dur = t.duration.unwrap_or(global_duration);
        if tapping {
            return (v, fallback_dur);
        }
    }

    // while_hover: active → use its value; inactive → remember its duration
    if let Some(h) = while_hover
        && let Some(v) = get(h)
    {
        fallback_dur = h.duration.unwrap_or(global_duration);
        if hovering {
            return (v, fallback_dur);
        }
    }

    // animate
    if let Some(b) = animate
        && let Some(v) = get(b)
    {
        return (v, b.duration.unwrap_or(global_duration));
    }

    (neutral, fallback_dur)
}

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
    /// Per-state duration override. When set, properties from this target
    /// use this duration instead of the global [`MotionBuilder::duration`].
    pub duration: Option<Duration>,
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

    /// Per-state duration override. Properties resolved from this target
    /// use this duration instead of the global [`MotionBuilder::duration`].
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
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
    pub while_hover: Option<PropertyTarget>,
    pub while_tap: Option<PropertyTarget>,
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
            while_hover: None,
            while_tap: None,
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

    // Shorthand methods — merge into existing animate target.

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

    pub fn while_hover(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
        self.while_hover = Some(f(PropertyTarget::default()));
        self
    }

    pub fn while_tap(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
        self.while_tap = Some(f(PropertyTarget::default()));
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
    builder: MotionBuilder,
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
        let builder = f(MotionBuilder::default());

        // Start from the initial state if one was provided, otherwise
        // use the neutral defaults.
        let init_opacity = builder
            .initial
            .as_ref()
            .and_then(|i| i.opacity)
            .unwrap_or(1.0);
        let init_x = builder.initial.as_ref().and_then(|i| i.x).unwrap_or(0.0);
        let init_y = builder.initial.as_ref().and_then(|i| i.y).unwrap_or(0.0);
        let init_scale = builder
            .initial
            .as_ref()
            .and_then(|i| i.scale)
            .unwrap_or(1.0);
        let init_rotate = builder
            .initial
            .as_ref()
            .and_then(|i| i.rotate)
            .unwrap_or(0.0);

        Motion {
            inner: self,
            builder,
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
    #[allow(clippy::too_many_arguments)]
    fn animate_property<T: gpui::Lerp + Clone + PartialEq + 'static>(
        &mut self,
        key_suffix: &str,
        target: T,
        start: T,
        base_id: &ElementId,
        duration: Duration,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> T {
        let key = ElementId::NamedChild(Arc::new(base_id.clone()), key_suffix.into());
        let easing = self.builder.easing;
        let t = window
            .use_keyed_transition(key, cx, duration, |_, _| start.clone())
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

        // Resolve active target by merging animate + interaction states.
        // Per Framer Motion: higher-priority states override specific properties;
        // properties not set in an active state fall through to animate.
        let offset = point(px(self.current_x), px(self.current_y));
        let tapping = read_tapping(&base_id);
        let hovering = read_hovering(&base_id);

        let while_tap = self.builder.while_tap.as_ref();
        let while_hover = self.builder.while_hover.as_ref();
        let animate = self.builder.animate.as_ref();
        let global_duration = self.builder.duration;

        let (target_opacity, opacity_dur) = resolve_property(
            tapping, while_tap, hovering, while_hover, animate,
            |t| t.opacity, 1.0, global_duration,
        );
        let (target_x, x_dur) = resolve_property(
            tapping, while_tap, hovering, while_hover, animate,
            |t| t.x, 0.0, global_duration,
        );
        let (target_y, y_dur) = resolve_property(
            tapping, while_tap, hovering, while_hover, animate,
            |t| t.y, 0.0, global_duration,
        );
        let (target_scale, scale_dur) = resolve_property(
            tapping, while_tap, hovering, while_hover, animate,
            |t| t.scale, 1.0, global_duration,
        );
        let (target_rotate, rotate_dur) = resolve_property(
            tapping, while_tap, hovering, while_hover, animate,
            |t| t.rotate, 0.0, global_duration,
        );

        // Evaluate each animated property and cache for paint.
        self.current_opacity = self.animate_property(
            "opacity", target_opacity, self.current_opacity, &base_id,
            opacity_dur, window, cx,
        );
        self.current_x = self.animate_property(
            "x", target_x, self.current_x, &base_id,
            x_dur, window, cx,
        );
        self.current_y = self.animate_property(
            "y", target_y, self.current_y, &base_id,
            y_dur, window, cx,
        );
        self.current_scale = self.animate_property(
            "scale", target_scale, self.current_scale, &base_id,
            scale_dur, window, cx,
        );
        self.current_rotate = self.animate_property(
            "rotate", target_rotate, self.current_rotate, &base_id,
            rotate_dur, window, cx,
        );

        // Keep the render loop alive when interaction states are configured,
        // otherwise animation frames stop once all transitions complete and
        // hover/tap detection stops working.
        if self.builder.while_hover.is_some() || self.builder.while_tap.is_some() {
            window.request_animation_frame();
        }

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

        // Interaction event listeners.
        let base_id = self.inner.id().unwrap();
        let has_hover = self.builder.while_hover.is_some();
        let has_tap = self.builder.while_tap.is_some();

        if has_hover || has_tap {
            let cells = get_or_init_cells(&base_id);

            // Hover detection via MouseMoveEvent.
            if has_hover {
                let h = cells.hovering.clone();
                window.on_mouse_event::<MouseMoveEvent>(
                    move |_: &MouseMoveEvent, phase, window, _| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }
                        let now = !window.last_input_was_keyboard()
                            && paint_bounds.contains(&window.mouse_position());
                        if now != h.get() {
                            h.set(now);
                        }
                    },
                );
            }

            // Tap detection via MouseDown/MouseUp.
            if has_tap {
                let t = cells.tapping.clone();
                window.on_mouse_event::<MouseDownEvent>(
                    move |e: &MouseDownEvent, phase, window, _| {
                        if phase != DispatchPhase::Bubble || e.button != MouseButton::Left {
                            return;
                        }
                        if paint_bounds.contains(&window.mouse_position()) {
                            t.set(true);
                        }
                    },
                );

                let t2 = cells.tapping;
                window.on_mouse_event::<MouseUpEvent>(
                    move |e: &MouseUpEvent, phase, _, _| {
                        if phase != DispatchPhase::Bubble || e.button != MouseButton::Left {
                            return;
                        }
                        t2.set(false);
                    },
                );
            }
        }

        window.with_element_opacity(opacity, |window| {
            // Paint inner element.
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

    // Motion element construction tests.
    //
    // Verify that MotionExt::motion() correctly initializes the Motion
    // struct's current_* fields from the builder's initial state, falling
    // back to neutral defaults when no initial is set.

    /// Without an `initial` state, all current_* fields use neutral defaults:
    /// opacity=1.0, x=0, y=0, scale=1.0, rotate=0.
    #[test]
    fn motion_construction_neutral_defaults() {
        use gpui::{div, InteractiveElement};

        let motion = div()
            .id("test")
            .motion(|m| m.animate(|s| s.opacity(0.5)));

        assert!((motion.current_opacity - 1.0).abs() < f32::EPSILON);
        assert!((motion.current_x - 0.0).abs() < f32::EPSILON);
        assert!((motion.current_y - 0.0).abs() < f32::EPSILON);
        assert!((motion.current_scale - 1.0).abs() < f32::EPSILON);
        assert!((motion.current_rotate - 0.0).abs() < f32::EPSILON);
    }

    /// When `initial` is set, current_* values match the initial state,
    /// not the animate target.
    #[test]
    fn motion_construction_initial_state_propagates() {
        use gpui::{div, InteractiveElement};

        let motion = div().id("test").motion(|m| {
            m.initial(|s| s.opacity(0.0).x(10.0).y(20.0).scale(0.5).rotate(45.0))
                .animate(|s| s.opacity(1.0).x(0.0).y(0.0))
        });

        // Current values come from initial.
        assert!((motion.current_opacity - 0.0).abs() < f32::EPSILON);
        assert!((motion.current_x - 10.0).abs() < f32::EPSILON);
        assert!((motion.current_y - 20.0).abs() < f32::EPSILON);
        assert!((motion.current_scale - 0.5).abs() < f32::EPSILON);
        // rotate stores radians internally: 45° = π/4.
        let expected_rotate = 45.0 * std::f32::consts::PI / 180.0;
        assert!((motion.current_rotate - expected_rotate).abs() < f32::EPSILON);
    }

    /// rotate_radians in initial state stores the value directly (no
    /// degree→radian conversion).
    #[test]
    fn motion_construction_rotate_radians_initial() {
        use gpui::{div, InteractiveElement};

        let motion = div().id("test").motion(|m| {
            m.initial(|s| s.rotate_radians(std::f32::consts::PI))
                .animate(|s| s.rotate_radians(0.0))
        });

        assert!((motion.current_rotate - std::f32::consts::PI).abs() < f32::EPSILON);
    }

    /// When only some initial properties are set, the rest fall back to
    /// neutral defaults — not to the animate target.
    #[test]
    fn motion_construction_partial_initial_falls_back_to_neutral() {
        use gpui::{div, InteractiveElement};

        // Only set opacity in initial.
        let motion = div()
            .id("test")
            .motion(|m| m.initial(|s| s.opacity(0.0)).animate(|s| s.opacity(1.0)));

        assert!((motion.current_opacity - 0.0).abs() < f32::EPSILON);
        assert!((motion.current_scale - 1.0).abs() < f32::EPSILON); // neutral
        assert!((motion.current_x - 0.0).abs() < f32::EPSILON); // neutral
        assert!((motion.current_y - 0.0).abs() < f32::EPSILON); // neutral
        assert!((motion.current_rotate - 0.0).abs() < f32::EPSILON); // neutral
    }

    // Shorthand + animate merging edge cases.

    /// Shorthand before animate: animate *replaces* the shorthand target.
    /// This is a documented sharp edge — shorthand and animate() are
    /// not composable in that order.
    #[test]
    fn shorthand_then_animate_overwrites() {
        let c = MotionBuilder::default()
            .opacity(0.5)
            .animate(|s| s.x(100.0));
        let a = c.animate.unwrap();
        // x was set by animate, but opacity from the earlier shorthand is gone.
        assert_eq!(a.opacity, None);
        assert_eq!(a.x, Some(100.0));
    }

    /// Shorthand after animate: shorthand *merges* into the existing
    /// animate target.
    #[test]
    fn animate_then_shorthand_merges() {
        let c = MotionBuilder::default()
            .animate(|s| s.opacity(0.5))
            .x(100.0)
            .y(200.0);
        let a = c.animate.unwrap();
        assert_eq!(a.opacity, Some(0.5));
        assert_eq!(a.x, Some(100.0));
        assert_eq!(a.y, Some(200.0));
    }

    /// Chaining only shorthands merges all of them into a single
    /// animate target.
    #[test]
    fn multiple_shorthands_merge() {
        let c = MotionBuilder::default()
            .opacity(0.3)
            .x(100.0)
            .y(200.0)
            .scale(1.5)
            .rotate(180.0);
        let a = c.animate.unwrap();
        assert_eq!(a.opacity, Some(0.3));
        assert_eq!(a.x, Some(100.0));
        assert_eq!(a.y, Some(200.0));
        assert_eq!(a.scale, Some(1.5));
        // rotate shorthand converts degrees→radians.
        let expected_rotate = 180.0 * std::f32::consts::PI / 180.0;
        assert!((a.rotate.unwrap() - expected_rotate).abs() < f32::EPSILON);
    }

    /// Calling animate twice: the second call replaces the first completely.
    #[test]
    fn animate_twice_replaces() {
        let c = MotionBuilder::default()
            .animate(|s| s.opacity(0.5).x(100.0))
            .animate(|s| s.y(200.0));
        let a = c.animate.unwrap();
        assert_eq!(a.opacity, None);
        assert_eq!(a.x, None);
        assert_eq!(a.y, Some(200.0));
    }

    /// Calling initial twice: the second call replaces the first.
    #[test]
    fn initial_twice_replaces() {
        let c = MotionBuilder::default()
            .initial(|s| s.opacity(0.0))
            .initial(|s| s.x(100.0));
        let i = c.initial.unwrap();
        assert_eq!(i.opacity, None);
        assert_eq!(i.x, Some(100.0));
    }

    #[test]
    fn while_hover_sets_field() {
        let c = MotionBuilder::default().while_hover(|s| s.scale(1.1));
        let h = c.while_hover.unwrap();
        assert_eq!(h.scale, Some(1.1));
    }

    #[test]
    fn while_tap_sets_field() {
        let c = MotionBuilder::default().while_tap(|s| s.scale(0.95));
        let t = c.while_tap.unwrap();
        assert_eq!(t.scale, Some(0.95));
    }

    #[test]
    fn all_states_chain() {
        let c = MotionBuilder::default()
            .animate(|s| s.opacity(1.0).scale(1.0))
            .while_hover(|s| s.scale(1.1))
            .while_tap(|s| s.scale(0.95))
            .duration(Duration::from_millis(200));

        assert_eq!(c.animate.unwrap().opacity, Some(1.0));
        assert_eq!(c.while_hover.unwrap().scale, Some(1.1));
        assert_eq!(c.while_tap.unwrap().scale, Some(0.95));
        assert_eq!(c.duration, Duration::from_millis(200));
    }
}
