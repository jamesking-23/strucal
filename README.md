# StruCal
Structural calculation and verification platform.

- `crates/engine` - deterministic Rust calculation engine (single source of truth)
- `crates/wasm` - engine compiled to WebAssembly for the web app (output committed to `apps/web/public/wasm`)
- `apps/web` - Next.js web app: landing page, workspace, project, standards and account screens
- `apps/desktop` - native Windows desktop app (egui; uses the engine directly, no webview)
- `supabase` - backend migrations and the assistant edge function

## Prerequisites
- Rust stable with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- `wasm-bindgen-cli` at the exact version in `Cargo.lock` (currently 0.2.130): `cargo install wasm-bindgen-cli --version 0.2.130 --locked`
- Node.js 22

## Engine
```bash
cargo test -p strucal-engine
```
The suite includes hand-calculation checks per module and a robustness sweep (`robustness_tests.rs`) that runs every module with extreme and malformed inputs and requires no panic and no non-finite result.

## Rebuild the WebAssembly engine
Required after any change under `crates/engine` or `crates/wasm`:
```bash
cargo build -p strucal-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/strucal_wasm.wasm --target web --out-dir apps/web/public/wasm --no-typescript
node scripts/wasm-smoke.mjs
```
`scripts/wasm-smoke.mjs [dir]` runs every module on its defaults through a WASM build and prints the trace hashes, so two builds can be diffed for behavioural parity. CI fails if the committed bundle disagrees with the engine source.

## Web app
```bash
cd apps/web
npm ci
npm run dev      # http://localhost:3000
npm run build && npm start
```
The design system (colour, type, spacing tokens, light and dark themes, shared components) lives in `apps/web/app/globals.css`. Inputs are kept in browser storage; local projects can be exported and imported as JSON.

## Desktop app
```bash
cargo run -p strucal-desktop
```
Windows releases are built by `.github/workflows/desktop.yml`. The palette mirrors the web tokens in `apps/desktop/src/main.rs`.

## Checks before pushing
```bash
cargo fmt --all --check
cargo test -p strucal-engine
cd apps/web && npx tsc --noEmit && npm run build
```
