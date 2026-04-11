# mkg-starship

A small **Rust + Bevy 0.18** 2D lunar-lander style game: procedural terrain with flat landing pads, three thrusters (main + two 45° diagonals), methane fuel, per-body gravity (Earth → Moon → Mars → Mercury), HUD, scoring, and a single crash before game over.

The design plan lives in [docs/PLAN.md](docs/PLAN.md).

## Controls

| Input | Action |
| ----- | ------ |
| **↓** | Main thruster (up) |
| **Z** | Left diagonal (up-right) |
| **/** | Right diagonal (up-left) |
| **R** | Restart (after game over) |

## Native (desktop)

```bash
cargo run --release
```

## WebAssembly (Chrome / any modern browser)

1. Install the WASM target (once):

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Build:

   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

3. Generate JS glue (requires [wasm-bindgen-cli](https://github.com/rustwasm/wasm-bindgen), e.g. `cargo install wasm-bindgen-cli`):

   ```bash
   wasm-bindgen target/wasm32-unknown-unknown/release/mkg-starship.wasm --out-dir wasm --target web --no-typescript
   ```

4. Serve the `wasm/` folder over HTTP (browsers block `file://` modules). For example:

   ```bash
   npx --yes serve wasm -p 8080
   ```

   Then open `http://localhost:8080` in Chrome.

**Alternative:** install [wasm-server-runner](https://github.com/jetli/wasm-server-runner) (`cargo install wasm-server-runner`) and use the [`.cargo/config.toml`](.cargo/config.toml) runner so `cargo run --target wasm32-unknown-unknown --release` starts a local server (you still need `wasm-bindgen` output if you rely on the static `wasm/` layout above).

Configure the server to send `Content-Type: application/wasm` for `.wasm` files when possible (many static servers do this automatically).

## License

See [LICENSE](LICENSE).
