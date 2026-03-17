# Copilot Instructions

## Build & Run

```sh
# Build
cargo build

# Run (requires a terminal — reads terminal size at startup)
cargo run

# Build release (as used in CI)
cargo build --release --locked
```

There are no tests in this project.

On Linux, `rdev` requires X11 dev libs: `sudo apt install pkg-config libx11-dev gcc-multilib`

## Architecture

Single-file Rust terminal snake game (`src/main.rs`). The entire game runs in one file with these key structs:

- **`GameState`** — central coordinator; owns `Field`, `Cobra`, score, level, and all shared state. `next_tick()` is the main game loop step.
- **`Field`** — the game board. Holds three separate `Vec<ThingOnScreen>`: `edges` (border walls), `things` (food/rocks/drugs), `cobra_things` (cobra segments re-generated each tick).
- **`Cobra`** — body stored as `Vec<Position>` ordered **tail-first, head-last** (index 0 = tail, last index = head). Movement appends a new head and removes the tail unless growing.
- **`ThingOnScreen`** — generic entity on the board; has a `value` string (render format), `position`, `kind`, and `effect`.
- **`CobraEffect`** — `Blow` (death), `Grow` (ate food), `PowerUp` (ate drug). When a `ThingOnScreen` is consumed, its `effect` is set to `None` to mark it as eaten.

### Threading model

Input is captured on a background thread via `rdev::listen`. Three global `once_cell::Lazy` statics are shared between threads:
- `INPUT_QUEUE: Arc<Mutex<Vec<EventType>>>` — key events pushed by input thread, popped by game loop
- `KEY_IS_PRESSED: Arc<Mutex<bool>>` — tracks held-key state for acceleration
- `TICK: Arc<SafeMutex<i32>>` — `parking_lot::Mutex` aliased as `SafeMutex`; the game loop holds this lock during sleep, allowing `show_game_over` to detect the pause via `is_locked()`

### Rendering

Uses the `utilprint` crate with a custom format string: `@COLOR#UNICODE_HEX`, e.g.:
- `@G#2501` = green + `━` (cobra body)
- `@Y#25C9` = yellow + `◉` (food)
- `@M#2605` = magenta + `★` (drug / powered-up state)
- `@W#2620` = white + `☠` (rock / edge collision)

### Level & speed

`Level::get_speed` returns `level_number * min_delay`. The game divides `1000ms` by this to get tick duration. Holding a direction key halves the delay (acceleration). `CobraState::PoweredUp` doubles speed for `u8::MAX` ticks.

## Key Conventions

- **Rust 2024 edition** — uses if-let chains (`if let Some(x) = ... && condition`).
- **`parking_lot::Mutex`** (imported as `SafeMutex`) is used only for `TICK` because it provides `try_lock_for(duration)` for timed sleeping and `is_locked()`.
- **`std::sync::Mutex`** is used for `INPUT_QUEUE` and `KEY_IS_PRESSED`.
- Things are consumed by setting `thing.effect = None`; checking `thing.effect.is_some()` is the canonical way to test if a thing is still active (e.g., `food_left()`).
- The cobra body order (tail→head) means `get_value(index)` determines segment appearance: no prev = tail, no next = head, both = body segment.
- Terminal dimensions are read once at startup via `terminal_size`; the field is `(h-3) × (w-2)` to leave room for the HUD line and borders.
