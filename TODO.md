# TODO List - Ravel (Toy JS Runtime)

## 🥇 Tier 0 (Highest Priority — if this breaks, everything breaks)

### Runtime Core

#### Event Loop
- [ ] Task queue / microtask queue
- [ ] Promise scheduling integration with event loop

#### Promise / Future bridge
- [ ] Bridge Rust Future ↔ JS Promise

#### microtask / macrotask consistency
- [ ] `queueMicrotask`
- [ ] Correct ordering between microtasks and macrotasks

#### Timer accuracy
- [ ] `setTimeout` / `setInterval` — Accurate timing
- [ ] `clearTimeout` / `clearInterval`

#### AbortController
- [ ] `AbortController` / `AbortSignal`

---

## 🥈 Tier 1 (The "Modern Runtime" Line)

### Web API First

#### fetch / Request / Response
- [ ] `fetch` — Use `reqwest` for HTTP
- [ ] `Request` / `Response` objects

#### Headers
- [ ] `Headers` class

#### URL / URLSearchParams
- [ ] `URL` class
- [ ] `URLSearchParams` class

### Module System

#### ESM only
- [ ] ES Modules (`import`/`export`) — Async dependency graph, QuickJS ESM hooks

#### module graph / cache
- [ ] Module graph resolution
- [ ] Module caching

---

## 🥉 Tier 2 (Viable as a CLI Runtime)

### Process
- [ ] `process.env` — Environment variables
- [ ] `process.argv` — Command-line arguments
- [ ] `process.exit()` — Force exit
- [ ] `process.cwd()` — Current working directory

### File System (Organize & Improve)

#### Promise-based unification
- [x] `fs.readFile` / `fs.writeFile` — Read/write files as `Uint8Array`
- [x] `fs.exists` — Check if a path exists
- [ ] Migrate all fs APIs to Promise-first

#### text helpers
- [ ] `fs.readTextFile` / `fs.writeTextFile` — String-based helpers

---

## 🏅 Tier 3 (Completeness — Making It Work Right)

### Scheduling

#### queueMicrotask
- [ ] `queueMicrotask` — Native implementation

#### Promise queue integration
- [ ] Unified promise queue with event loop

#### setImmediate (optional)
- [ ] `setImmediate` / `clearImmediate`

### Binary / Streams

#### TextEncoder / Decoder
- [ ] `TextEncoder` / `TextDecoder`

#### Blob
- [ ] `Blob` class

#### Streams
- [ ] Web Streams API (`ReadableStream`, `WritableStream`, `TransformStream`)

---

## 🎯 Tier 4 (DX — Developer Experience)

### TypeScript

#### transpile (oxc)
- [ ] Add `oxc_parser` — Parse TS/JS source into AST
- [ ] Add `oxc_transformer` — Strip TS types, transform to ES2015+ JS
- [ ] Add `oxc_codegen` — Generate JS string from transformed AST
- [ ] `--ts` flag — Auto-detect or force TypeScript input
- [ ] `.ts` file support in REPL and file execution

#### check command (delegated externally)
- [ ] Implement `ravel check` command:
  - Use Rust `std::process::Command` to run `tsgo` in the background
  - Parse type errors and display them in a clean format
  - Show installation prompt if `tsgo` is not installed
- [ ] `ravel check` options:
  - `--strict` — Run type checking in strict mode
  - `--no-emit` — Type check only (default)
  - `--watch` — Watch for file changes and re-check continuously
- [ ] Auto-detect and support `tsconfig.json`

### CLI / REPL

#### Error formatting
- [ ] Format errors with stack traces, code frames, and hints

#### source map
- [ ] Source map support for transpiled output

#### REPL improvements
- [ ] `process.stdin` — Interactive input for CLI tools
- [ ] REPL improvements (history, tab completion, syntax highlighting)

---

## Archive (Completed)

- [x] `fs.readFile` / `fs.writeFile` — Read/write files as `Uint8Array`
- [x] `fs.exists` — Check if a path exists
- [x] `__dirname` / `__filename` — Current file directory and path
