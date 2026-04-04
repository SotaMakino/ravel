# ravel

A toy JavaScript runtime written in Rust.

## Architecture

Ravel uses a **QuickJS-first** strategy, leveraging Rust's Tokio ecosystem to provide a high-performance asynchronous foundation.

- **Engine**: Full ES6+ compliance via QuickJS (Arrow functions, Classes, Destructuring, Promises, etc.)
- **Async Runtime**: Tokio-based event loop integrated into the QuickJS context
  - `setTimeout` / `setInterval` exposed to the global scope
  - JavaScript Promises bridged with Rust Futures
- **Standard Library**: `Math`, `JSON`, `Date` provided natively by QuickJS
- **Filesystem**: Sandboxed `fs` module (`readFile`, `writeFile`, `exists`) scoped to the script's directory
- **High-performance `console.log`** via Rust FFI

## Usage

```bash
# Run a file
ravel file.js

# Show help
ravel --help

# Start a REPL
ravel
```

## Building

```bash
cargo build
```

## QuickJS

Uses Bellard's QuickJS engine via the `rquickjs` crate. Full ES6+ support with Rust-native async runtime.

**Language Features:**
- Arrow functions, template literals, classes, destructuring, spread/rest
- `Promise`, `try/catch`, async/await
- `Math`, `JSON`, `Date` — provided natively by QuickJS
- `Object.keys/values`, `Array.map/filter/reduce/find`, `String.toUpperCase/split/includes/substring`

**Async Runtime:**
- Tokio-based event loop
- `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval` in global scope
- Promise/Future bridging for Rust async interop

**Standard Library (Rust FFI):**
- `console.log` — Custom Rust callback
- `fs.readFile(path)` — Reads file as Uint8Array, sandboxed to script directory
- `fs.writeFile(path, data)` — Writes Uint8Array to file, sandboxed to script directory
- `fs.exists(path)` — Checks if file exists, sandboxed to script directory

**Globals:**
- `__filename` — Absolute path of the running script
- `__dirname` — Directory of the running script

## REPL

Run `ravel` with no arguments to start the interactive REPL. Features:

- Line history (up/down arrows)
- History persisted between sessions
- Timer support (REPL waits for pending timers)

## Project Structure

```
src/
  core/               — QuickJS low-level wrapper and execution foundation
    mod.rs            — Runtime definition, context management
    engine.rs         — Bytecode execution, memory limit configuration
    event_loop.rs     — Tokio and JS Promise integration
  console.rs          — console.log
  fs.rs               — fs.readFile, writeFile, exists
  timer.rs            — setTimeout, setInterval, clearTimeout, clearInterval
  cli/                — Command-line interface
  lib.rs              — Crate entry point
  main.rs             — Binary entry point
examples/
  basic.js          — Basic JavaScript features demo
  timers.js         — Timer features demo
  fs.js             — Filesystem features demo
  sandbox.js        — Sandbox boundary demo
tests/
  integration_test.rs — Integration tests
```

## Tests

```bash
cargo test
```
