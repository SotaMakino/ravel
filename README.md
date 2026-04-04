# ravel

A toy JavaScript/TypeScript runtime written in Rust.

## Usage

```bash
ravel file.js      # run JavaScript
ravel file.ts      # run TypeScript (auto-transpiled via Oxc)
ravel              # start REPL
ravel --help
```

## Building

```bash
cargo build
```

## Tests

```bash
cargo test
```

## Features

- **ES6+** via QuickJS — promises, async/await, classes, modules, etc.
- **TypeScript** — `.ts`/`.tsx` files are stripped of types by Oxc and fed directly into QuickJS
- **Async runtime** — `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval` on a Tokio event loop
- **Sandboxed fs** — `fs.readFile`, `fs.writeFile`, `fs.exists` scoped to the script's directory
- **ESM imports** — relative and bare imports with `.js`, `.mjs`, `.ts`, `.tsx` resolution
- **Globals** — `__filename`, `__dirname`
- **REPL** — line history with persistence, timer support

## Project Structure

```
src/
  core/         — QuickJS runtime, module loader, event loop
  transpiler.rs — Oxc TypeScript-to-JavaScript transpiler
  console.rs    — console.log
  fs.rs         — sandboxed filesystem
  timer.rs      — setTimeout / setInterval
  cli/          — CLI entry point
examples/       — usage demos
tests/          — integration + snapshot tests
```
