# cobrinha

A terminal-based snake game written in Rust.

## Requirements

- Rust toolchain (2024 edition)
- Linux: X11 development libraries

```sh
sudo apt install pkg-config libx11-dev gcc-multilib
```

## Build and run

```sh
cargo build --release
cargo run
```

## Controls

| Key         | Action              |
|-------------|---------------------|
| Arrow keys  | Change direction    |
| Hold arrow  | Accelerate          |
| R           | Restart (game over) |
| Q           | Quit                |

## Gameplay

Eat all the food on the field to advance to the next level. Each level adds more food and rocks, and the cobra moves faster.

| Symbol | Item  | Effect                                      |
|--------|-------|---------------------------------------------|
| Food   | Eat   | Grows the cobra, advances level when cleared |
| Star   | Drug  | Powers up the cobra and doubles speed temporarily |
| Skull  | Rock  | Kills the cobra                             |
| Border | Wall  | Kills the cobra                             |

The cobra starts with 3 lives. Running into a rock, the border, or itself costs one life. Losing all lives ends the game.

Score increases each tick and on level completion. Scoring is multiplied by level and doubled while powered up.

## License

This project is unlicensed.
