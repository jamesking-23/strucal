# StruCal
Structural calculation and verification platform.

- `crates/engine` - deterministic Rust calculation engine (single source of truth)
- `crates/wasm` - engine compiled to WebAssembly for the web app
- `apps/web` - Next.js web app (landing page + workspace)
- `apps/desktop` - native Windows desktop app (uses the engine directly, no webview)

Rebuild WASM: `cargo build -p strucal-wasm --target wasm32-unknown-unknown --release && wasm-bindgen target/wasm32-unknown-unknown/release/strucal_wasm.wasm --target web --out-dir apps/web/public/wasm --no-typescript`
