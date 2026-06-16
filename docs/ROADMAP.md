# gpui-motion Roadmap

12-phase roadmap, from core animation engine to cross-platform integration.

Status markers used in each phase:
- ✅ Done
- 🟡 In progress
- 📋 Planned
- ❌ Blocked (GPUI platform constraint)

---

## Phase 0 — Core Engine (P0)

Core animation driver + declarative Element API.

| Feature | Status | Notes |
|---------|--------|-------|
| Easing curves (32 presets + custom) | ✅ | `src/easing.rs`, 15 tests |
| `Tween<T>` state machine | ✅ | `src/tween.rs`, 10 tests |
| `.motion()` declarative API | ✅ | `src/motion.rs`, 20 tests |
| opacity animation | ✅ | Via `MotionBuilder` + `PropertyTarget` |
| x / y translation animation | ✅ | Via `window.with_element_offset` |
| scale animation | ✅ | Via `TransformationMatrix` |
| rotate animation | ✅ | Via `TransformationMatrix` |
| initial → animate two-state | ✅ | Entrance animation support |
| Driven by `gpui::Transition` | ✅ | No duplicate animation engine |
| border_radius / color / shadow | 📋 | Simple plumbing, pending |
| Spring physics engine | 📋 | Currently easing-only |
| FLIP layout animation | 📋 | Auto-animate position/size changes |

**Phase 0 remaining work:**
- Spring physics → see Phase 7
- FLIP → see Phase 3
- More visual properties → see Phase 6

---

## Phase 1 — Declarative States (P0)

Declarative animation state management.

| Feature | Status | Notes |
|---------|--------|-------|
| AnimatePresence (exit animation) | 📋 | GPUI constraint #8: must use wrapper model |
| Variants (named multi-state) | 📋 | Architecture analysis: `if/else` + `Transition` more natural in GPUI; may not be needed |
| whileHover / whileTap / whileFocus | 📋 | Interaction state animations |
| imperative `controls.start()` | 📋 | Imperative playback control |

**Design notes:**
- AnimatePresence cannot detect VDOM removal like React; must use `observe_release` + keyed child wrapper
- Variants may be dropped entirely in favor of `if/else` + `Animated<T>`

---

## Phase 2 — Gestures (P1)

Drag/gesture-driven animation.

| Feature | Status | Notes |
|---------|--------|-------|
| Drag (drag-to-move) | 📋 | GPUI constraint #7: `on_drag` is drag-and-drop |
| Pan (pan gesture) | 📋 | Possibly via `PinchGestureEvent` |
| Focus animation | 📋 | Focus-change triggered animation |
| Multi-pointer | ❌ | GPUI constraint #6: single-touch only |

---

## Phase 3 — Layout & Reorder (P1)

Layout animation and list reordering.

| Feature | Status | Notes |
|---------|--------|-------|
| FLIP layout animation | 📋 | Auto-interpolate position/size changes |
| layoutId shared layout | 📋 | Cross-page magic move |
| Reorder.Group | 📋 | Drag-to-sort list |
| Grid/masonry reorder | 📋 | |

---

## Phase 4 — SVG & Path (P2)

SVG path animation.

| Feature | Status | Notes |
|---------|--------|-------|
| SVG path morphing | 📋 | `d` attribute interpolation |
| SVG stroke/fill animation | 📋 | |
| pathLength offset | 📋 | Line-drawing animation |
| Path following | 📋 | Element movement along a path |

---

## Phase 5 — Custom Values (P2)

Generic animated value system.

| Feature | Status | Notes |
|---------|--------|-------|
| `Animated<T>` generic value | 📋 | User-definable interpolation types |
| useTransform | 📋 | Map one value to another |
| useSpring | 📋 | Attach spring physics to any value |
| useVelocity | 📋 | Track value velocity |

---

## Phase 6 — Visual Properties (P2)

Expand the set of animatable visual properties.

| Feature | Status | Notes |
|---------|--------|-------|
| border_radius animation | 📋 | Via `with_corner_radii` |
| background color animation | 📋 | Needs sRGB + HSL color space interpolation |
| box-shadow animation | 📋 | GPUI constraint #2: no element post-processing |
| filter / backdrop-filter | ❌ | GPUI constraint #2 |
| clip-path | ❌ | GPUI constraint #3: rectangle only |
| gradient animation | 📋 | Linear gradient support |
| color (foreground) | 📋 | `text_color` property |

---

## Phase 7 — Physics Drivers (P2)

Physics-driven and advanced animation controls.

| Feature | Status | Notes |
|---------|--------|-------|
| Spring physics | 📋 | Mass/stiffness/damping parameters |
| Inertia / Decay | 📋 | Inertial decay animation |
| Keyframes | 📋 | Multi-keyframe animation |
| Repeat / Loop / Yoyo | 📋 | |
| Time control (speed/pause) | 📋 | |

---

## Phase 8 — Scroll & Viewport (P3)

Scroll-driven animation.

| Feature | Status | Notes |
|---------|--------|-------|
| scroll-linked animation | 📋 | Scroll position drives animation progress |
| viewport detection | 📋 | Trigger on element enter/leave viewport |
| parallax | 📋 | |

---

## Phase 9 — Orchestration (P3)

Multi-element animation orchestration.

| Feature | Status | Notes |
|---------|--------|-------|
| stagger | 📋 | Staggered delay sequence |
| sequence | 📋 | `animate([a, b, c])` |
| when (conditional) | 📋 | Chain: one completes → next starts |
| GroupAnimation | 📋 | Group-level orchestration |

---

## Phase 10 — 3D Transforms (P3)

Three-dimensional transforms.

| Feature | Status | Notes |
|---------|--------|-------|
| translateZ / rotateX / rotateY | 📋 | GPUI constraint #1: currently 2×3 affine only |
| perspective | 📋 | Extensible via wgpu |
| 3D flip card | 📋 | |

---

## Phase 11 — Performance & Tools (P3)

Performance optimization and developer tooling.

| Feature | Status | Notes |
|---------|--------|-------|
| `prefers-reduced-motion` | ❌ | GPUI constraint #5: platform not exposed |
| pause / resume global control | 📋 | |
| seek / scrub | 📋 | Timeline scrubbing |
| Animation inspector | 📋 | Visual debugging |
| Batch update optimization | 📋 | Coalesce state changes within a frame |

---

## Phase 12 — Cross-platform & Integration (P3)

Cross-platform and ecosystem integration.

| Feature | Status | Notes |
|---------|--------|-------|
| WASM / Web support | 📋 | |
| GPU-accelerated animation | 📋 | Via wgpu |
| gpui-component integration | 📋 | Interop with component library |

---

## Priority Overview

| Priority | Phases | Key items |
|----------|--------|-----------|
| 🔥 P0 | 0, 1 | AnimatePresence, complete Phase 0 properties |
| 🔥 P1 | 2, 3 | Drag, Layout animation |
| ⭐ P2 | 4, 5, 6, 7 | SVG, Custom values, Visual properties, Spring |
| 💤 P3 | 8, 9, 10, 11, 12 | Scroll, Orchestration, 3D, Tools, Integration |

## GPUI Platform Constraints (non-bypassable)

1. 3D transforms — currently only 2×3 affine matrix; extensible via wgpu
2. CSS filter / backdrop-filter — no element-wide post-processing pipeline
3. Non-rectangular clip-path — `with_content_mask` is rectangle-only
4. Radial gradients / mask-image
5. `prefers-reduced-motion` — not exposed by platform
6. Multi-touch — only surface `PinchGestureEvent`
7. `on_drag` naming conflict — GPUI uses it for drag-and-drop, not drag-to-move
8. No element unmount hook — dictates AnimatePresence must use wrapper model

---

> **Last updated:** 2026-06-16
> **Maintainer:** @mengh04
