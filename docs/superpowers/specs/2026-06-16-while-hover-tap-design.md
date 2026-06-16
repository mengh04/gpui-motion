# whileHover / whileTap Interaction State Animation

**Date:** 2026-06-16  
**Status:** Design approved

## Overview

Add `while_hover` and `while_tap` interaction states to `MotionBuilder`, enabling animated property transitions triggered by mouse hover and mouse press.

Model: **replacement** — when an interaction state is active, its `PropertyTarget` replaces the base `animate` target. Framer Motion's priority order applies: **tap > hover > animate**.

## API Surface

```rust
// Base animation only (unchanged)
.motion(|m| m
    .animate(|s| s.opacity(1.0))
)

// With hover state — scale up on hover
.motion(|m| m
    .animate(|s| s.opacity(1.0))
    .while_hover(|s| s.scale(1.1))
)

// With tap state — scale down on press
.motion(|m| m
    .animate(|s| s.opacity(1.0))
    .while_hover(|s| s.scale(1.1))
    .while_tap(|s| s.scale(0.95))
)

// Shared duration/easing for all states (all transitions share the same curve)
.motion(|m| m
    .animate(|s| s.opacity(1.0))
    .while_hover(|s| s.scale(1.1))
    .while_tap(|s| s.scale(0.95))
    .duration(Duration::from_millis(200))
    .easing(Easing::EaseOutCubic)
)
```

`duration` and `easing` are shared across all states (animate, while_hover, while_tap). This matches Framer Motion's shared transition model.

## Data Structures

### MotionBuilder changes

```rust
pub struct MotionBuilder {
    pub initial: Option<PropertyTarget>,
    pub animate: Option<PropertyTarget>,
    pub while_hover: Option<PropertyTarget>,   // NEW
    pub while_tap: Option<PropertyTarget>,     // NEW
    pub duration: Duration,
    pub easing: Easing,
}

impl MotionBuilder {
    pub fn while_hover(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
        self.while_hover = Some(f(PropertyTarget::default()));
        self
    }
    pub fn while_tap(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
        self.while_tap = Some(f(PropertyTarget::default()));
        self
    }
}
```

### MotionInteractionState (persistent)

Stored via `window.with_element_state()`, keyed by `(GlobalElementId, TypeId::of::<MotionInteractionState>())`. Survives element recreation across frames.

```rust
struct MotionInteractionState {
    hovering: Rc<Cell<bool>>,
    tapping: Rc<Cell<bool>>,
}
```

`Rc<Cell<bool>>` enables shared ownership between the persistent state and per-frame event listeners. Listeners clone the `Rc`, write to the `Cell` on events, and the next frame's prepaint reads the latest value.

### Motion<E> itself

No new fields. The struct is unchanged. Interaction state is accessed via `with_element_state` during prepaint/paint.

## Runtime Flow

### prepaint

```
1. Retrieve MotionInteractionState via window.with_element_state()
   - First frame: create new (hovering=false, tapping=false)
2. Read hovering.get() and tapping.get()
3. Resolve active target:
   if tapping.get() → builder.while_tap
   elif hovering.get() → builder.while_hover
   else → builder.animate
4. If active target has changed to a different PropertyTarget,
   or individual property values within the target changed,
   the existing animate_property() mechanism automatically
   transitions to new values via gpui::Transition
```

### paint

```
1. Apply opacity/transform wrappers as before
2. window.insert_hitbox(bounds) → Hitbox
3. Clone hovering/tapping Rc from state, register event listeners:

   // MouseMove: update hover only when NOT tapping (Framer deferred hover)
   window.on_mouse_event::<MouseMoveEvent>(|_, _, window, _| {
       if !tapping.get() {
           hovering.set(hitbox.is_hovered(window));
       }
   });

   // MouseDown on element starts tap
   window.on_mouse_event::<MouseDownEvent>(|_, _, window, _| {
       if hitbox.is_hovered(window) {
           tapping.set(true);
       }
   });

   // MouseUp anywhere: end tap, re-evaluate hover
   window.on_mouse_event::<MouseUpEvent>(|_, _, window, _| {
       tapping.set(false);
       hovering.set(hitbox.is_hovered(window));
   });

4. Paint inner element normally
```

### Interaction behavior (matches Framer Motion)

- **Hover**: active while pointer is over the element. Pointer leave ends hover.  
  *During press:* hover is frozen — `MouseMoveEvent` does NOT update `hovering` while `tapping` is true. This matches Framer's deferred hover logic (`hover.ts:98-103`).
- **Tap**: active from pointer-down on element until pointer-up anywhere. Moving outside element during press keeps tap active — only release ends it.
- **MouseUpEvent**: sets `tapping = false`, then re-evaluates `hovering` via `hitbox.is_hovered(window)`. This resolves the deferred hover state.
- **Priority**: tap > hover > animate. When multiple states are active, the higher-priority one wins.

## Files to Modify

| File | Change |
|------|--------|
| `src/motion.rs` | Add `while_hover`/`while_tap` fields to `MotionBuilder`. Add builder methods. Add `MotionInteractionState`. Modify `prepaint` for target resolution. Modify `paint` for hitbox + event listeners. |
| `src/motion.rs` (tests) | Add tests for builder methods, priority resolution, default interaction state. |

## Edge Cases

1. **No while_hover/while_tap set**: behavior unchanged — always uses animate target
2. **Only some properties in while_hover**: like `initial`, only the specified properties animate; others stay at their current values
3. **while_hover without animate**: hover properties transition from neutral defaults
4. **Element without .id()**: `MotionExt` already requires `Stateful<E>` (has `.id()`), so this is guaranteed
5. **Framer Motion: press moves outside**: tap stays active, confirmed from source
6. **Framer Motion: hover during press**: hover persists, deferred until pointerup, confirmed from source

## Open Questions

- `HitboxBehavior` — use `Normal` (default) to not interfere with inner element interactivity
- Speed of `MouseMoveEvent` handler — `hitbox.is_hovered()` is a cheap bounds check, no concern
