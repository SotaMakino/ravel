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
- **JSX rendering** — `.tsx` files transform JSX into `note()` calls that produce HTML strings
- **Async runtime** — `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval` on a Tokio event loop
- **Sandboxed fs** — `fs.readFile`, `fs.writeFile`, `fs.exists` scoped to the script's directory
- **ESM imports** — relative and bare imports with `.js`, `.mjs`, `.ts`, `.tsx` resolution
- **Globals** — `__filename`, `__dirname`
- **REPL** — line history with persistence, timer support

## Project Structure

```
src/
  core/         — QuickJS runtime, module loader, event loop
  transpiler.rs — Oxc TypeScript-to-JavaScript transpiler (with JSX support)
  jsx.rs        — note() HTML renderer (Hono-style JSX runtime)
  console.rs    — console.log
  fs.rs         — sandboxed filesystem
  timer.rs      — setTimeout / setInterval
  cli/          — CLI entry point
examples/       — usage demos
tests/          — integration + snapshot tests
```

## JSX Support

Write `.tsx` files with JSX syntax and they are automatically transpiled and executed:

```tsx
// example.tsx
const el = <div class="card"><h1>Hello</h1></div>;
console.log(el);
// Output: <div class="card"><h1>Hello</h1></div>

function Badge(props) {
  return <span class="badge">{props.text}</span>;
}
console.log(<Badge text="New" />);
// Output: <span class="badge">New</span>
```

JSX is transformed using the classic runtime with `note` as the pragma function. The `note()` function is injected into the QuickJS global scope and renders JSX to HTML strings with support for:
- Elements with attributes
- Nested/children elements
- Self-closing void tags (`<br>`, `<img>`, etc.)
- Fragments (`<>...</>`)
- Function components with props
- Attribute value escaping
