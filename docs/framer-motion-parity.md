# gpui-motion ↔ Framer Motion Parity

Feature-by-feature comparison between gpui-motion and [Framer Motion](https://www.framer.com/motion/).

Legend:
- **Status**: ✅ Done / 🟡 Partial / 📋 Planned / ❌ Blocked by platform
- **Parity**: High (API close) / Medium (concept equivalent, different impl) / Low (large differences) / None (unsupported)

---

## §1 Animation Engine

Core value interpolation and frame scheduling.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| Easing curves | 32+ cubic-bezier presets + custom | 32 presets + `CubicBezier(x1,y1,x2,y2)` | ✅ | High | easing.rs, 15 tests |
| Tween driver | `animate({ x: 100 }, { duration: 0.3 })` | `Tween::new(from, to, duration, easing)` | ✅ | High | tween.rs, 10 tests |
| Spring driver | `animate({ x: 100 }, { type: "spring" })` | - | 📋 | None | Planned Phase 7 |
| Inertia/Decay | `animate({ x: 100 }, { type: "inertia" })` | - | 📋 | None | Planned Phase 7 |
| Keyframes | `animate({ x: [0, 100, 50] })` | - | 📋 | None | Planned Phase 7 |
| Frame scheduler | Global shared `requestAnimationFrame` | `gpui::Transition` internal scheduling | ✅ | Medium | Provided by GPUI, not self-managed |
| Subscriber model | `MotionValue.on("change", cb)` | `gpui::Transition` auto-notifies | ✅ | Medium | Different GPUI model, equivalent outcome |
| Lifecycle callbacks | 6 (`onPlay`/`onUpdate`/`onComplete`/…) | - | 📋 | None | gpui::Transition doesn't expose callbacks |
| Playback controls | `controls.play()`/`pause()`/`stop()` | - | 📋 | None | Planned Phase 11 |
| Group / Sequence | `animate([a, b, c])` / `staggerChildren` | - | 📋 | None | Planned Phase 9 |
| Repeat / Loop / Yoyo | `repeat: Infinity`, `repeatType: "mirror"` | - | 📋 | None | Planned Phase 7 |

---

## §2 MotionValue System

Generic animated value abstraction.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `MotionValue<T>` generic | `const x = useMotionValue(0)` | `gpui::Transition<T>` equivalent | ✅ | Medium | Reuses GPUI built-in |
| `.get()` / `.set()` | `x.get()` / `x.set(100)` | `transition.update()` | ✅ | Medium | |
| `.on("change", cb)` | Event subscription | - | 📋 | Low | GPUI uses entity observation |
| useTransform | `const y = useTransform(x, v => v * 2)` | - | 📋 | None | Planned Phase 5 |
| useSpring | `const x = useSpring(value, { stiffness: 100 })` | - | 📋 | None | Planned Phase 5 |
| useVelocity | `const v = useVelocity(x)` | - | 📋 | None | Planned Phase 5 |

---

## §3 Declarative Element API

Declaring animations on elements.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `motion.div` | `<motion.div animate={{ x: 100 }} />` | `div().id("x").motion(\|m\| m.x(100.))` | ✅ | High | Declarative style matches |
| `initial` | `initial={{ opacity: 0 }}` | `.initial(\|s\| s.opacity(0.))` | ✅ | High | |
| `animate` | `animate={{ opacity: 1 }}` | `.animate(\|s\| s.opacity(1.))` | ✅ | High | |
| `exit` | `exit={{ opacity: 0 }}` | - | 📋 | None | Requires AnimatePresence |
| `whileHover` | `whileHover={{ scale: 1.1 }}` | - | 📋 | None | Planned Phase 1 |
| `whileTap` / `whileFocus` | Interaction states | - | 📋 | None | Planned Phase 1 |
| `variants` | `variants={{ open: {...}, closed: {...} }}` | - | 📋 | Low | Architecture analysis: `if/else` more natural in GPUI |
| `transition` | `transition={{ duration: 0.5, ease: "easeInOut" }}` | `.duration(...).easing(...)` | ✅ | High | |
| `style` prop-driven | `<motion.div style={{ x }} />` | - | 📋 | None | Requires `Animated<T>` value API |
| Element types | div, span, path, svg… | Any type implementing `Element` | ✅ | High | Generic `Motion<E>` |

**Supported property mapping (5/15):**

| Property | Framer Motion | gpui-motion | Status |
|----------|-------------|-------------|--------|
| opacity | ✅ | ✅ | ✅ |
| x | ✅ | ✅ | ✅ |
| y | ✅ | ✅ | ✅ |
| scale | ✅ | ✅ | ✅ |
| rotate | ✅ | ✅ | ✅ |
| scaleX / scaleY | ✅ | - | 📋 |
| rotateX / rotateY / rotateZ | Z-axis ≈ rotate | - | 📋 (3D: Phase 10) |
| skew | ✅ | - | 📋 |
| translateZ | ✅ | - | 📋 (3D) |
| borderRadius | ✅ | - | 📋 (Phase 6) |
| backgroundColor | ✅ | - | 📋 (Phase 6) |
| color | ✅ | - | 📋 (Phase 6) |
| boxShadow | ✅ | - | 📋 (Phase 6) |
| width / height | `layout` animation | - | 📋 (Phase 3: FLIP) |
| pathLength / pathOffset | ✅ | - | 📋 (Phase 4: SVG) |

---

## §4 Gestures

Gesture-driven animation.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| hover detection | `whileHover` | GPUI: `on_hover` | 📋 | Medium | GPUI has native events |
| tap / press | `whileTap` | GPUI: `on_click` | 📋 | Medium | |
| focus | `whileFocus` | GPUI: `on_focus` | 📋 | Medium | |
| drag (drag-to-move) | `<motion.div drag />` | - | 📋 | None | GPUI `on_drag` is drag-and-drop |
| pan | `onPan` | `PinchGestureEvent` | 📋 | Low | |
| multi-pointer | Multi-touch gestures | - | ❌ | None | GPUI constraint #6 |
| dragConstraints | `dragConstraints={{ left: 0 }}` | - | 📋 | None | |
| dragElastic | `dragElastic={0.2}` | - | 📋 | None | |
| dragTransition | Release snap-back animation | - | 📋 | None | |

---

## §5 Layout Animation (FLIP)

Auto-animation of layout changes.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `layout` prop | `<motion.div layout />` | - | 📋 | None | Planned Phase 3 |
| `layoutId` shared layout | `<motion.div layoutId="x" />` | - | 📋 | None | Magic move |
| `layoutScroll` | Follow parent scroll | - | 📋 | None | |
| FLIP calculation | First-Last-Invert-Play | - | 📋 | None | |

---

## §6 AnimatePresence

Mount/unmount animation.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `AnimatePresence` | `<AnimatePresence> {children} </AnimatePresence>` | - | 📋 | None | P0, design pending |
| `exit` animation | `exit={{ opacity: 0 }}` | - | 📋 | None | |
| `onExitComplete` | Exit-complete callback | - | 📋 | None | |
| Multi-element exit order | Key-driven exit queue | - | 📋 | None | |
| `mode="wait"` | Exit first, then enter | - | 📋 | None | |
| `mode="sync"` | Simultaneous enter/exit | - | 📋 | None | |
| `mode="popLayout"` | Others move into place on exit | - | 📋 | None | |

**Architecture constraint:** GPUI has no VDOM unmount hook. AnimatePresence must use parent wrapper + `observe_release` model, not Framer's per-element `exit` prop.

---

## §7 Reorder & Drag-Sort

List drag-to-reorder.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `Reorder.Group` | `<Reorder.Group values={items} onReorder={...} />` | - | 📋 | None | |
| `Reorder.Item` | `<Reorder.Item value={item} />` | - | 📋 | None | |
| Drag swap | Position-swap animation between two items | - | 📋 | None | |
| List enter/exit | Animation on list changes | - | 📋 | None | |

---

## §8 SVG & Path

SVG path animation.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `motion.path` | `<motion.path animate={{ d: "..." }} />` | - | 📋 | None | |
| `motion.circle` / `motion.rect` | SVG shape elements | - | 📋 | None | |
| pathLength | `animate={{ pathLength: 1 }}` | - | 📋 | None | Line-drawing animation |
| pathOffset | `animate={{ pathOffset: [0, 1] }}` | - | 📋 | None | |
| pathSpacing | `animate={{ pathSpacing: 1 }}` | - | 📋 | None | |

---

## §9 Visual Properties

Animatable visual style properties (beyond the 5 core properties in §3).

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| border_radius | ✅ | - | 📋 | - | Phase 6 |
| background-color | ✅ | - | 📋 | - | Needs color space interpolation |
| color (foreground) | ✅ | - | 📋 | - | |
| box-shadow | ✅ | - | 📋 | - | Needs custom interpolator |
| text-shadow | ✅ | - | 📋 | - | |
| filter (`blur`/`brightness`/…) | ✅ | - | ❌ | None | GPUI constraint #2 |
| backdrop-filter | ✅ | - | ❌ | None | GPUI constraint #2 |
| clip-path | ✅ | - | ❌ | None | GPUI constraint #3 (rectangle only) |
| gradient (linear) | ✅ | - | 📋 | - | |
| gradient (radial) | ✅ | - | ❌ | None | GPUI constraint #4 |

---

## §10 Scroll & Viewport

Scroll-position and viewport-driven animation.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `useScroll` | `const { scrollY } = useScroll()` | - | 📋 | None | Phase 8 |
| scroll-linked | Scroll position maps to animation value | - | 📋 | None | |
| `useInView` | `const isInView = useInView(ref)` | - | 📋 | None | |
| viewport margin | `viewport={{ once: true, margin: "-100px" }}` | - | 📋 | None | |

---

## §11 Orchestration

Multi-element / multi-animation orchestration.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `delayChildren` | Variant child delay | - | 📋 | None | Phase 9 |
| `staggerChildren` | Variant child stagger | - | 📋 | None | |
| `staggerDirection` | Stagger direction | - | 📋 | None | |
| `when: "beforeChildren"` | Parent before children | - | 📋 | None | |
| `when: "afterChildren"` | Children before parent | - | 📋 | None | |
| sequence | `animate([a, b, c])` | - | 📋 | None | |

---

## §12 Accessibility & Global Config

Accessibility and global configuration.

| Feature | Framer Motion | gpui-motion | Status | Parity | Notes |
|---------|-------------|-------------|--------|--------|-------|
| `MotionConfig` | `<MotionConfig reducedMotion="user" />` | - | 📋 | None | |
| `prefers-reduced-motion` | System-level preference | - | ❌ | None | GPUI constraint #5 |
| `useReducedMotion` | Hook query | - | ❌ | None | GPUI constraint #5 |
| Global duration/easing defaults | `<MotionConfig transition={{ duration: 0.5 }} />` | - | 📋 | None | |
| `MotionGlobalConfig` | Module-level global config (`skipAnimations`) | - | 📋 | None | |

---

## §13 API Ergonomics (Rust-isms)

Rust-specific API design considerations.

| Aspect | Framer Motion | gpui-motion | Status | Notes |
|--------|-------------|-------------|--------|-------|
| Type-safe properties | JS: runtime checks | Rust: compile-time checks | ✅ | Inherent advantage |
| Builder pattern | JS: mutable objects | `.motion(\|m\| m.x(100.).duration(...))` | ✅ | |
| `.id()` requirement | No requirement | Must call `.id()` before `.motion()` | ✅ | GPUI Transition needs key |
| Generic element support | JS: dynamic types | `Motion<E: Element>`, any element works | ✅ | |
| Error messages | JS: runtime exceptions | Rust: compile-time errors | ✅ | |
| Performance | JS: GC pressure | Rust: zero-cost abstractions | ✅ | |

---

## §14 Priority Next Steps

Ordered by priority.

### Immediate (P0)

1. **AnimatePresence** — Biggest P0 gap, unlocks exit animations
   - Design: parent wrapper + `observe_release` + keyed child
   - Risk: GPUI has no unmount hook, design must be careful

2. **More animatable properties** — border_radius, background color
   - Low risk, straightforward plumbing
   - Color interpolation needs color space selection (sRGB, HSL, Oklch)

3. **`Animated<T>` generic value API** — Let users compose and use animation values
   - Shape similar to Compose `animateDpAsState`
   - Reuse `gpui::Transition<T>`

### Later (P1)

4. **Spring physics** — More natural animation feel
5. **FLIP layout animation** — Auto-animate position/size changes
6. **Drag gesture** — Drag-to-move support

### Long-term (P2+)

7. SVG path animation
8. Orchestration (stagger/sequence)
9. 3D transforms
10. Cross-platform (WASM)

---

> **Last updated:** 2026-06-16
> **Maintainer:** @mengh04
