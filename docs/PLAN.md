# Bevy 2D WASM landing game

## Context

This repository uses a **single binary crate** at the repo root.

## Architecture

```mermaid
flowchart LR
  subgraph core [Core loop]
    FixedUpdate[Fixed timestep physics]
    Update[Render and input]
  end
  subgraph systems [Systems]
    TerrainGen[terrain generation]
    ShipInput[thruster input]
    Collide[collision and landing]
    Progress[planet and score]
    HUD[HUD text]
  end
  FixedUpdate --> ShipInput
  FixedUpdate --> Collide
  Update --> HUD
  Collide --> Progress
  TerrainGen --> Collide
```

- **State**: Bevy `States` (`AppState`: `Playing`, `GameOver`) so input and physics ignore dead states cleanly.
- **Physics**: Integrate in **`FixedUpdate`** with constant `dt`. Store **velocity** (`Vec2`) on the ship; each step apply gravity, then thruster accelerations when keys are held and fuel > 0.
- **Terrain**: Piecewise-linear height profile — ordered `(x, y)` points. Mountains from random walk + noise; **2–3 flat segments** are landing pads.
- **Collision**: Sample terrain height at ship foot `x`. **Crash** if not on a pad or impact speed too high. **Safe landing** on a pad below speed thresholds → points, refill fuel, next planet (or victory after Mercury).
- **Rendering**: `Mesh2d` for terrain polygon; `Mesh2d` + `Rectangle` for ship.

## Thrusters and fuel

- **Center** (Arrow Down): thrust straight up — slows descent.
- **Left** (Z): thrust up-right at 45°.
- **Right** (Slash `/`): thrust up-left at 45°.
- **Fuel**: Single tank; drains while any thruster fires.

## Planets and gravity

| Phase | Body    | Gravity (m/s²) |
| ----- | ------- | -------------- |
| 1     | Earth   | 9.81           |
| 2     | Moon    | 1.62           |
| 3     | Mars    | 3.71           |
| 4     | Mercury | 3.70           |

Gravity is applied as **scaled game acceleration** (see `planets.rs` / `constants.rs`).

## HUD

Upper-right UI: methane (fuel), horizontal and vertical velocity.

## WASM build

- Target: `wasm32-unknown-unknown`
- `getrandom` with `wasm_js` on wasm; `console_error_panic_hook` in `main`
- Bevy features: `2d`, `web`, `png` (see `Cargo.toml`)
- Serve over HTTP (e.g. `wasm-server-runner`, `trunk`, or static server). See [README.md](../README.md).

## Modules

| File          | Role                                              |
| ------------- | ------------------------------------------------- |
| `main.rs`     | App, plugins, state, wasm panic hook              |
| `camera.rs`   | Camera / world bounds                             |
| `terrain.rs`  | Generation, mesh, height queries                  |
| `ship.rs`     | Ship component, spawning                          |
| `physics.rs`  | Gravity, thrusters, integration                   |
| `collision.rs`| Landing vs crash                                  |
| `game_flow.rs`| Score, planet progression, restart                |
| `ui.rs`       | HUD                                               |
| `planets.rs`  | Planet ordering and gravity constants             |

## Game rules

- **One crash**: First crash → `GameOver` (failed run).
- **Points**: Base per landing + bonus for lower speed.
- **Victory**: Successful landing on Mercury ends the run in a win state.

## Deliverables

1. Native: `cargo run`
2. WASM: `cargo build --target wasm32-unknown-unknown --release` + HTTP serve
3. Playable loop: Earth → Moon → Mars → Mercury; HUD; fuel; three thrusters; procedural terrain with pads; single life.
