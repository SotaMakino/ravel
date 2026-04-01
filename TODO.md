# TODO List - Ravel (Toy JS Runtime)

## Architecture

### Standard Runtime (`--jsc`, Default) ✅
- **Full ES6+** via JavaScriptCore — Arrow functions, template literals, classes, destructuring, spread, promises, try/catch
- **Tokio-based async runtime** integrated into JSC context
  - `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval` in global scope
  - JavaScript Promises bridged with Rust Futures
- **Standard Library (Rust FFI)**
  - `console.log` — Custom Rust callback via `function_callback`
  - `Math` — `random`, `floor`, `ceil`, `abs`, `max`, `min`, `pow`, `sqrt`, constants
  - `JSON` — `stringify`, `parse`
  - `Date` — `now`, `getTimestamp`

### Manual Mode (`--manual`) ✅
- **Core language** — Variables, operators, control flow, functions, closures, objects, arrays
- **`console.log`** — Rust callback
- **Timers** — `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval` (tokio-based)
- **Role**: Experimental sandbox for language design experiments

## TODO

### TypeScript Support
- [ ] Add `oxc_parser` — Parse TS/JS source into AST
- [ ] Add `oxc_transformer` — Strip TS types, transform to ES2015+ JS
- [ ] Add `oxc_codegen` — Generate JS string from transformed AST
- [ ] `--ts` flag — Auto-detect or force TypeScript input
- [ ] `.ts` file support in REPL and file execution

### Step 2: Type Checking (The Guard)
- [ ] Implement `ravel check` command:
  - Use Rust `std::process::Command` to run `tsgo` in the background
  - Parse type errors and display them in a clean format
  - Show installation prompt if `tsgo` is not installed
- [ ] `ravel check` options:
  - `--strict` — Run type checking in strict mode
  - `--no-emit` — Type check only (default)
  - `--watch` — Watch for file changes and re-check continuously
- [ ] Auto-detect and support `tsconfig.json`

### Runtime Essentials (to become a real runtime like Node/Bun)

#### File System
- [ ] `fs.readFile` / `fs.writeFile` — Wrap `std::fs` for JS access
- [ ] `fs.readdir`, `fs.stat`, `fs.mkdir`, `fs.unlink`
- [ ] `path.join`, `path.resolve`, `__dirname`, `__filename`

#### Networking
- [ ] `fetch` / `Request` / `Response` — Use `reqwest` for HTTP
- [ ] TCP/UDP sockets — For building servers

#### Module System
- [ ] CommonJS (`require`) — Sync file loading with caching
- [ ] ES Modules (`import`/`export`) — Async dependency graph, JSC ESM hooks

#### Process
- [ ] `process.env` — Environment variables
- [ ] `process.argv` — Command-line arguments
- [ ] `process.exit()` — Force exit
- [ ] `process.cwd()` — Current working directory

#### Binary Data
- [ ] `Buffer` utilities — Base64, hex encoding/decoding
- [ ] `Uint8Array` extensions for efficient binary I/O

#### Stdio
- [ ] `process.stdin` — Interactive input for CLI tools

### Event Loop
- [ ] Task queue / microtask queue
- [ ] `queueMicrotask`
- [ ] `setImmediate`

### Manual Backend Enhancements
- [ ] Arrow functions
- [ ] Template literals
- [ ] Classes
- [ ] Destructuring
- [ ] Spread/rest operators
- [ ] try/catch
- [ ] Promise support
- [ ] Array methods (`map`, `filter`, `reduce`, etc.)
- [ ] Object methods (`keys`, `values`, etc.)
- [ ] String methods (`toUpperCase`, `split`, etc.)
