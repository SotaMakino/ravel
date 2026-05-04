# ravel

A toy JavaScript/TypeScript runtime written in Rust.

## Usage

```bash
ravel file.js      # run JavaScript
ravel file.ts      # run TypeScript (auto-transpiled via Oxc)
ravel              # start REPL
ravel --build file.js  # SSG build mode (one-off, no timers)
ravel --serve      # serve dist/ directory on port 3000
ravel --serve 8080    # serve dist/ directory on custom port
ravel --help
ravel --version
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
- **Sandboxed fs** — `fs.readFile`, `fs.writeFile` (auto-creates parent dirs), `fs.mkdirSync`, `fs.exists` scoped to the script's directory; path traversal, symlink escape, and null byte attacks are blocked
- **ESM imports** — relative and bare imports with `.js`, `.mjs`, `.ts`, `.tsx` resolution
- **Globals** — `__filename`, `__dirname`, `process.env`, `ravel.version`, `ravel.build`
- **REPL** — line history with persistence, timer support
- **SSG build mode** — `--build` flag runs scripts as one-off compilation tasks with `ravel.build === true` and `process.env.RAVEL_BUILD === "1"`
- **Dev server** — `--serve` flag serves the `dist/` directory over HTTP (default port 3000, configurable)

## Project Structure

```
src/
  core.rs       — module declarations for core/
  core/         — QuickJS runtime, module loader, event loop
  transpiler.rs — Oxc TypeScript-to-JavaScript transpiler (with JSX support)
  jsx.rs        — note() HTML renderer (Hono-style JSX runtime)
  console.rs    — console.log
  fs.rs         — sandboxed filesystem
  timer.rs      — setTimeout / setInterval
  cli.rs        — module declarations for cli/
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

## SSG Build Mode

Use `--build` to run a script as a one-off static site generation task. Unlike normal execution, build mode:

- Sets `ravel.build` to `true` (vs `false` in normal/REPL mode)
- Sets `process.env.RAVEL_BUILD` to `"1"`
- Skips the timer event loop (no persistent runtime)

```js
// build.js
if (ravel.build) {
  console.log("Running in build mode");
  fs.writeFile("dist/index.html", new TextEncoder().encode("<h1>Built</h1>"));
}
```

```bash
ravel --build build.js
```

## Dev Server

Serve the output of `--build` over HTTP:

```bash
ravel --serve          # serve dist/ on port 3000
ravel --serve 8080     # serve dist/ on port 8080
```

The server looks for a `dist/` directory in the current working directory and serves its contents. Unknown paths fall back to `dist/index.html` for SPA-style routing.

## Build Metadata

The following globals are available in all modes:

| Global | Description |
|--------|-------------|
| `ravel.version` | Current ravel version (e.g. `"0.3.0"`) |
| `ravel.build` | `true` when run with `--build`, `false` otherwise |
| `process.env` | Object containing all environment variables |
| `process.env.RAVEL_BUILD` | `"1"` in build mode, `undefined` otherwise |

```js
console.log(ravel.version);  // "0.3.0"
console.log(process.env.HOME);  // "/Users/..."
```

## Enhanced Filesystem

`fs.writeFile` automatically creates parent directories, making it suitable for writing to nested static routes without manual directory setup:

```js
// Writes to dist/posts/hello/index.html, creating all intermediate directories
const html = new TextEncoder().encode("<h1>Hello</h1>");
fs.writeFile("dist/posts/hello/index.html", html);
```

`fs.mkdirSync(path)` creates directories recursively within the sandbox:

```js
fs.mkdirSync("dist/assets/css");  // creates dist/assets/css
fs.mkdirSync("dist");             // no-op if already exists
```
