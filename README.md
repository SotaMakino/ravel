# ravel

A toy JavaScript runtime written in Rust.

## Architecture

Ravel uses a **JavaScriptCore-first** strategy, leveraging Rust's Tokio ecosystem to provide a high-performance asynchronous foundation.

### Standard Runtime (`--jsc`, Default)

The primary execution environment integrating JSC with a native Rust-based host.

- **Engine**: Full ES6+ compliance via JavaScriptCore (Arrow functions, Classes, Destructuring, Promises, etc.)
- **Async Runtime**: Tokio-based event loop integrated into the JSC context
  - `setTimeout` / `setInterval` exposed to the global scope
  - JavaScript Promises bridged with Rust Futures
- **Standard Library**: Native support for `Math`, `JSON`, `Date`, etc. via Rust FFI
- **High-performance `console.log`** via Rust FFI (`function_callback`)

### Manual Mode (`--manual`)

A scratch-built, experimental interpreter used for internal prototyping and educational purposes.

- **Role**: Sub-feature for language design experiments and AST verification
- **Current State**: Supports core variables, closures, control flow, and legacy timer implementations
- **Goal**: Maintain as a lightweight sandbox for testing new runtime logic before JSC integration

## Usage

```bash
# Run a file (uses JavaScriptCore backend by default)
ravel file.js

# Run with manual interpreter (experimental)
ravel --manual file.js

# Start a REPL
ravel
```

## Building

```bash
# Default build (JavaScriptCore + Tokio async runtime)
cargo build

# Build with manual interpreter backend
cargo build --features manual
```

## Backend Details

### JavaScriptCore (Default)

Uses Apple's JavaScriptCore engine via the `javascriptcore` crate. Full ES6+ support with Rust-native async runtime.

**Language Features:**
- Arrow functions, template literals, classes, destructuring, spread/rest
- `Promise`, `try/catch`, async/await
- `Math`, `JSON`, `Date`, `Object.keys/values`, `Array.map/filter/reduce/find`, `String.toUpperCase/split/includes/substring`

**Async Runtime:**
- Tokio-based event loop
- `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval` in global scope
- Promise/Future bridging for Rust async interop

**Standard Library (Rust FFI):**
- `console.log` — Custom Rust callback
- `Math` — `random`, `floor`, `ceil`, `abs`, `max`, `min`, `pow`, `sqrt`, constants (`PI`, `E`, etc.)
- `JSON` — `stringify`, `parse`
- `Date` — `now`, `getTimestamp`

### Manual Interpreter (Optional)

A from-scratch JS interpreter written in Rust. Enable with `--features manual`. Supports a subset of JavaScript:

- **Variables** — `let`, `const`, `var` declarations and reassignment
- **Types** — numbers, strings, booleans, `null`, `undefined`, objects, arrays
- **Operators** — arithmetic, comparison, equality, logical
- **Control flow** — `if/else`, `while`, `for`
- **Functions** — declarations, calls, return, closures
- **Data structures** — object literals with property access, array literals with indexing
- **`console.log`** — builtin output
- **Timers** — `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval` (tokio-based)

## REPL

Run `ravel` with no arguments to start the interactive REPL. Features:

- Line history (up/down arrows)
- History persisted between sessions
- Timer support (REPL waits for pending timers)

## Project Structure

```
src/
  jsc/              — JavaScriptCore enhancements
    mod.rs          — Module organization & environment setup
    timers.rs       — Tokio-based setTimeout/setInterval for JSC
    promises.rs     — JS Promise / Rust Future bridging
    stdlib.rs       — Standard library (Math, JSON, Date) via Rust FFI
  lexer/            — tokenizer (manual backend)
  parser/           — recursive descent parser + AST (manual backend)
  interpreter.rs    — tree-walk interpreter (manual backend)
  builtins.rs       — builtin functions (manual backend)
  env.rs            — lexical environment (manual backend)
  value.rs          — runtime value types (manual backend)
  timer.rs          — timer state management (manual backend)
  main.rs           — CLI entry point
  lib.rs            — library root
```

## Tests

```bash
# Default tests (JavaScriptCore)
cargo test

# All tests including manual backend
cargo test --features manual
```
