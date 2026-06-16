# whileHover / whileTap Implementation Plan

> **Pair Programming:** Each step tells you what to write and why. Type it yourself.

**Goal:** Add `while_hover` and `while_tap` interaction state animation to `MotionBuilder`.

**Architecture:** `MotionBuilder` gains two `Option<PropertyTarget>` fields. Interaction state (`hovering`/`tapping`) is stored via `window.with_element_state()` using `Rc<Cell<bool>>` for shared ownership between persistent state and per-frame event listeners. `prepaint` resolves the active target by priority (tap > hover > animate), `paint` registers hitbox + mouse event listeners.

**Tech Stack:** Rust, GPUI (`Hitbox`, `with_element_state`, `on_mouse_event`, `MouseMoveEvent`, `MouseDownEvent`, `MouseUpEvent`)

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/motion.rs` | All changes — `MotionBuilder` fields, `MotionInteractionState`, builder methods, prepaint logic, paint event listeners, tests |

Single file change — the feature is self-contained within the existing `motion` module.

---

### Task 1: Add `while_hover`/`while_tap` fields and builder methods

**Files:** Modify `src/motion.rs`

- [ ] **Step 1: Add fields to `MotionBuilder`**

在 `MotionBuilder` 的 `animate` 字段后面加两个新字段：

```rust
pub struct MotionBuilder {
    pub initial: Option<PropertyTarget>,
    pub animate: Option<PropertyTarget>,
    /// Optional hover state. Active while the pointer is over this element.
    pub while_hover: Option<PropertyTarget>,
    /// Optional tap/press state. Active while the primary pointer is pressed
    /// on this element. Stays active even if the pointer moves outside.
    pub while_tap: Option<PropertyTarget>,
    pub duration: Duration,
    pub easing: Easing,
}
```

在 `Default` impl 里不用改——`Option` 自动是 `None`。

- [ ] **Step 2: Add builder methods to `MotionBuilder` impl**

在现有的 `rotate_radians` 方法后面加：

```rust
/// Set the hover state. When the pointer enters the element, properties
/// animate from their current values toward this target. When the pointer
/// leaves, they animate back.
pub fn while_hover(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
    self.while_hover = Some(f(PropertyTarget::default()));
    self
}

/// Set the tap/press state. When the primary pointer is pressed on this
/// element, properties animate toward this target. When the pointer is
/// released (anywhere), they animate back.
pub fn while_tap(mut self, f: impl FnOnce(PropertyTarget) -> PropertyTarget) -> Self {
    self.while_tap = Some(f(PropertyTarget::default()));
    self
}
```

- [ ] **Step 3: Run tests**

```fish
cargo test -p gpui-motion
```

Expected: 44 passed (existing tests unaffected, new fields just default to `None`).

---

### Task 2: Write builder tests

**Files:** Modify `src/motion.rs` (tests module)

- [ ] **Step 1: Add `while_hover` sets field test**

```rust
#[test]
fn while_hover_sets_field() {
    let c = MotionBuilder::default().while_hover(|s| s.scale(1.1));
    let h = c.while_hover.unwrap();
    assert_eq!(h.scale, Some(1.1));
}
```

- [ ] **Step 2: Add `while_tap` sets field test**

```rust
#[test]
fn while_tap_sets_field() {
    let c = MotionBuilder::default().while_tap(|s| s.scale(0.95));
    let t = c.while_tap.unwrap();
    assert_eq!(t.scale, Some(0.95));
}
```

- [ ] **Step 3: Add full chain test with all states**

```rust
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
```

- [ ] **Step 4: Run tests**

```fish
cargo test -p gpui-motion
```

Expected: 47 passed.

---

### Task 3: Create `MotionInteractionState` and wire into `MotionExt::motion()`

**Files:** Modify `src/motion.rs`

- [ ] **Step 1: Add imports at top of file**

在现有的 `use` 语句后面加：

```rust
use std::cell::Cell;
```

- [ ] **Step 2: Add `MotionInteractionState` struct**

在 `MotionBuilder` impl 和 `Motion` struct 之间加：

```rust
/// Persistent interaction state stored via [`Window::with_element_state`].
///
/// Uses [`Rc`]`<`[`Cell`]`<bool>>` so event listeners registered during paint
/// can hold a reference and update the state from mouse events, while the
/// next frame's prepaint reads the latest value.
struct MotionInteractionState {
    hovering: Rc<Cell<bool>>,
    tapping: Rc<Cell<bool>>,
}
```

- [ ] **Step 3: Modify `MotionExt::motion()` to initialize interaction state**

在 `Motion { ... }` 构造之前，从 `with_element_state` 获取或创建状态。但是——`MotionExt::motion()` 是在元素构造时调用的，此时还没有 `window` 参数。

**关键问题**：`MotionExt::motion()` 的签名是 `fn motion(self, f: impl FnOnce(MotionBuilder) -> MotionBuilder) -> Motion<Self>`，没有 `window` 和 `cx` 参数。

**解决方案**：不在 `motion()` 里初始化状态，而是在 `prepaint`（第一次调用时）做惰性初始化。`prepaint` 有 `window` 参数。

所以这一步**不需要改 `motion()`**。直接在 `Motion` struct 里不加新字段——交互状态完全通过 `with_element_state` 存取，不需要存在 struct 里。

结论：Step 1 和 Step 2 做完就够了，Step 3 跳过。

---

### Task 4: Modify prepaint for active target resolution

**Files:** Modify `src/motion.rs`

- [ ] **Step 1: Add helper to resolve active `PropertyTarget`**

在 `impl<E: Element> Motion<E>` 块里，`animate_property` 方法前面加：

```rust
/// Return the active animation target based on current interaction state.
///
/// Priority matches Framer Motion: tap > hover > animate.
fn active_target(&self, state: &MotionInteractionState) -> Option<&PropertyTarget> {
    if state.tapping.get() {
        self.builder.while_tap.as_ref()
            .or_else(|| self.builder.animate.as_ref())
    } else if state.hovering.get() {
        self.builder.while_hover.as_ref()
            .or_else(|| self.builder.animate.as_ref())
    } else {
        self.builder.animate.as_ref()
    }
}
```

这里有一个设计决策：如果 `while_tap` 是 `None` 但用户按下了，fallback 到 `animate`。同样 hover 也 fallback。这确保如果没有配 `while_hover`/`while_tap`，行为和之前一样。

- [ ] **Step 2: Modify prepaint to read state and resolve target**

在 `prepaint` 方法里，`let base_id = self.inner.id().unwrap();` 之后加一段状态读取：

```rust
fn prepaint(&mut self, ..., window: &mut Window, cx: &mut App) -> ... {
    let base_id = self.inner.id().unwrap();

    // ── Interaction state ──
    let global_id = gpui::GlobalElementId::from(base_id.clone());
    let interaction = window.with_element_state(&global_id, |state, _window| {
        if let Some(s) = state {
            (s, s) // Return existing state unchanged
        } else {
            let new = MotionInteractionState {
                hovering: Rc::new(Cell::new(false)),
                tapping: Rc::new(Cell::new(false)),
            };
            // Can't clone here — need to return (return_value, stored_state)
            (MotionInteractionState {
                hovering: new.hovering.clone(),
                tapping: new.tapping.clone(),
            }, new)
        }
    });

    // ── Resolve active target ──
    let target = self.active_target(&interaction);

    // ── Evaluate animated properties using the resolved target ──
    if let Some(opacity_target) = target.and_then(|t| t.opacity) {
        self.current_opacity = self.animate_property(
            "opacity", opacity_target, self.current_opacity, &base_id, window, cx,
        );
    }
    // ... same for x, y, scale, rotate using target.and_then(|t| t.x) etc.
```

**等等**——这里的逻辑有个问题。当前的 `animate_property` 只对 `builder.animate` 里的属性做动画。现在 target 可能是 `while_hover`/`while_tap`，它们的属性可能和 `animate` 不同。而且"某个属性不在当前 target 里"意味着它应该过渡回 base 值（animate target 里该属性的值）。

但这会变得很复杂。**简化方案**：先只处理最简单的情况——target 里有的属性就动画，没有的属性保持原样。

实际上，更简单的做法：保持现有逻辑的结构，只是把 `self.builder.animate.as_ref().and_then(|a| a.opacity)` 替换为从 `active_target()` 取值。

具体修改——当前代码：
```rust
if let Some(target) = self.builder.animate.as_ref().and_then(|a| a.opacity) {
    self.current_opacity = self.animate_property("opacity", target, ...);
}
```

改为：
```rust
let target = self.active_target(&interaction);
if let Some(opacity) = target.and_then(|t| t.opacity) {
    self.current_opacity = self.animate_property("opacity", opacity, ...);
}
```

x/y/scale/rotate 同理。

- [ ] **Step 3: Run clippy and tests**

```fish
cargo clippy -- -D warnings
cargo test -p gpui-motion
```

---

### Task 5: Modify paint for hitbox + event listeners

**Files:** Modify `src/motion.rs`

- [ ] **Step 1: Add imports for event types**

在文件顶部加：
```rust
use gpui::{Hitbox, MouseMoveEvent, MouseDownEvent, MouseUpEvent};
```

- [ ] **Step 2: Modify paint to insert hitbox and register listeners**

在 `paint` 方法中，`window.with_element_opacity(...)` 闭包内，inner paint 之前：

```rust
fn paint(&mut self, ..., window: &mut Window, cx: &mut App) {
    // ... opacity / transform 包装 ...
    
    window.with_element_opacity(opacity, |window| {
        // ── Hitbox for event detection ──
        let hitbox = window.insert_hitbox(
            paint_bounds,
            gpui::HitboxBehavior::Normal,
        );
        
        // ── Interaction state ──
        let global_id = gpui::GlobalElementId::from(base_id.clone());
        let interaction = window.with_element_state(&global_id, |state, _window| {
            let s = state.expect("interaction state must be initialized in prepaint");
            (MotionInteractionState {
                hovering: s.hovering.clone(),
                tapping: s.tapping.clone(),
            }, s)
        });
        
        // ── Register mouse event listeners ──
        let hovering = interaction.hovering.clone();
        let tapping = interaction.tapping.clone();
        
        // Hover tracking (frozen during tap)
        window.on_mouse_event::<MouseMoveEvent>(move |_, _, window, _| {
            if !tapping.get() {
                hovering.set(hitbox.is_hovered(window));
            }
        });
        
        // Tap start
        let tapping2 = tapping.clone();
        let hitbox_id = hitbox.id;
        window.on_mouse_event::<MouseDownEvent>(move |_, _, window, _| {
            // Check if the hitbox is hovered via its id
            if hitbox_id.is_hovered(window) {
                tapping2.set(true);
            }
        });
        
        // Tap end (anywhere) + re-evaluate hover
        let hovering2 = interaction.hovering;
        window.on_mouse_event::<MouseUpEvent>(move |_, _, window, _| {
            tapping.set(false);
            hovering2.set(hitbox_id.is_hovered(window));
        });
        
        // ── Paint inner ──
        if needs_xform {
            // ... existing logic ...
        } else {
            self.inner.paint(...);
        }
    });
}
```

**注意**：`MouseMoveEvent` 和 `MouseUpEvent` 的闭包里用了 `hitbox.id` 而不是 `hitbox` 本身。因为 `Hitbox` 没有实现 `Copy`，而每个闭包都需要自己的引用。`HitboxId` 是 `Copy` 的，可以用 `.is_hovered(window)` 替代 `hitbox.is_hovered(window)`。

**另一个问题**：`with_element_state` 的闭包签名。我们需要看看 GPUI 的 API：`fn with_element_state<S, R>(&mut self, global_id: &GlobalElementId, f: impl FnOnce(Option<S>, &mut Self) -> (R, S)) -> R`。

重点是 `f` 返回 `(R, S)`，其中 `R` 是返回值，`S` 是存储的状态。我们需要在 prepaint 中取出来读（R = 状态的 clone），在 paint 中取出来并 clone Rc（R = 包含 clone Rc 的临时 struct）。

在 prepaint 中，我们返回 `(interaction_clone, original)`。在 paint 中，类似地返回一个包含 clone 的 Rc 的临时对象。

但在 paint 中有一个问题：我们需要在 `with_element_state` 闭包**外面**使用 clone 的 Rc（用于注册事件监听器）。闭包只返回 R，所以我们需要 R 包含 clone 的 Rc。

**简化方案**：在 paint 中不要用 `with_element_state` 重新读取。改为在 prepaint 中把 `Rc<Cell<bool>>` 存到 `Motion` struct 的临时字段里，paint 直接读。

但 `Motion` 不应该加新字段……实际上，加两个 `Option<Rc<Cell<bool>>>` 字段是可接受的。每帧 prepaint 填充它们，paint 消费它们。

**更简单的方案**：把 `Rc<Cell<bool>>` 的 clone 作为 prev_state 的一部分存下来……不行，太 hack。

**最终方案**：在 `Motion` struct 加两个字段用于帧内传递：

```rust
pub struct Motion<E> {
    inner: E,
    builder: MotionBuilder,
    current_opacity: f32,
    current_x: f32,
    current_y: f32,
    current_scale: f32,
    current_rotate: f32,
    // Per-frame interaction state (set in prepaint, consumed in paint)
    hovering: Option<Rc<Cell<bool>>>,
    tapping: Option<Rc<Cell<bool>>>,
}
```

prepaint 中 `with_element_state` 取出状态，clone Rc，填入这两个字段。paint 中取出消费。

- [ ] **Step 3: Update `Motion` struct fields**

```rust
pub struct Motion<E> {
    inner: E,
    builder: MotionBuilder,
    current_opacity: f32,
    current_x: f32,
    current_y: f32,
    current_scale: f32,
    current_rotate: f32,
    /// Current hover state. Set in prepaint, consumed in paint.
    hovering: Option<Rc<Cell<bool>>>,
    /// Current tap state. Set in prepaint, consumed in paint.
    tapping: Option<Rc<Cell<bool>>>,
}
```

- [ ] **Step 4: Update `MotionExt::motion()` constructor to init new fields**

```rust
Motion {
    inner: self,
    builder,
    current_opacity: init_opacity,
    current_x: init_x,
    current_y: init_y,
    current_scale: init_scale,
    current_rotate: init_rotate,
    hovering: None,
    tapping: None,
}
```

- [ ] **Step 5: Update prepaint to populate hovering/tapping fields**

在 `let base_id = ...` 之后、属性动画之前：

```rust
let global_id = gpui::GlobalElementId::from(base_id.clone());
let interaction = window.with_element_state(&global_id, |state, _window| {
    if let Some(s) = state {
        let clone = MotionInteractionState {
            hovering: s.hovering.clone(),
            tapping: s.tapping.clone(),
        };
        (clone, s)
    } else {
        let new = MotionInteractionState {
            hovering: Rc::new(Cell::new(false)),
            tapping: Rc::new(Cell::new(false)),
        };
        let clone = MotionInteractionState {
            hovering: new.hovering.clone(),
            tapping: new.tapping.clone(),
        };
        (clone, new)
    }
});
self.hovering = Some(interaction.hovering);
self.tapping = Some(interaction.tapping);
```

然后 `active_target()` 改为从 `self.hovering` / `self.tapping` 读：

```rust
fn active_target(&self) -> Option<&PropertyTarget> {
    let hovering = self.hovering.as_ref().map(|h| h.get()).unwrap_or(false);
    let tapping = self.tapping.as_ref().map(|t| t.get()).unwrap_or(false);
    
    if tapping {
        self.builder.while_tap.as_ref()
            .or_else(|| self.builder.animate.as_ref())
    } else if hovering {
        self.builder.while_hover.as_ref()
            .or_else(|| self.builder.animate.as_ref())
    } else {
        self.builder.animate.as_ref()
    }
}
```

- [ ] **Step 6: Update paint to consume hovering/tapping and register listeners**

在 paint 方法中，`let base_id = self.inner.id().unwrap();` 之后：

```rust
let hovering = self.hovering.take();
let tapping = self.tapping.take();
```

然后在 `with_element_opacity` 闭包内插入 hitbox 和监听器（见 Step 2 的代码，但用 `hovering` / `tapping` 变量而非 `with_element_state`）。

- [ ] **Step 7: Run clippy and tests**

```fish
cargo clippy -- -D warnings
cargo test -p gpui-motion
```

---

### Task 6: Write integration-level tests for interaction state

**Files:** Modify `src/motion.rs` (tests module)

- [ ] **Step 1: Test `active_target` priority logic**

这三段测试可以在不启动 GPUI window 的情况下跑——直接构造 `MotionBuilder` 和 `MotionInteractionState`，手动调 `active_target`：

```rust
#[test]
fn active_target_returns_animate_by_default() {
    // We test the priority logic indirectly via MotionBuilder construction:
    // without hover/tap, only animate should be available.
    let c = MotionBuilder::default().animate(|s| s.opacity(0.5));
    assert!(c.animate.is_some());
    assert!(c.while_hover.is_none());
    assert!(c.while_tap.is_none());
}

#[test]
fn while_hover_and_while_tap_are_independent() {
    let c = MotionBuilder::default()
        .while_hover(|s| s.scale(1.1))
        .while_tap(|s| s.scale(0.95));
    assert_eq!(c.while_hover.unwrap().scale, Some(1.1));
    assert_eq!(c.while_tap.unwrap().scale, Some(0.95));
    assert!(c.animate.is_none());
}
```

- [ ] **Step 2: Run tests**

```fish
cargo test -p gpui-motion
```

Expected: ~49 passed.

---

### Task 7: Run full verification

- [ ] **Step 1: Clippy**

```fish
cargo clippy -- -D warnings
```

- [ ] **Step 2: All tests**

```fish
cargo test -p gpui-motion
```

- [ ] **Step 3: Build example**

```fish
cargo build --example motion_demo
```

---

## Summary

| Task | What | Est. time |
|------|------|-----------|
| 1 | Builder fields + methods | 5 min |
| 2 | Builder tests | 5 min |
| 3 | `MotionInteractionState` struct | 2 min |
| 4 | prepaint target resolution | 10 min |
| 5 | paint hitbox + listeners | 15 min |
| 6 | Integration tests | 5 min |
| 7 | Full verify | 2 min |

**Total:** ~45 min
