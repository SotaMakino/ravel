# ravel

A toy JavaScript runtime written in Rust with two execution backends.

## Usage

```bash
# Run a file (defaults to JavaScriptCore backend)
ravel file.js

# Use the manual interpreter
ravel --manual file.js

# Start a REPL
ravel
```

## Backends

### JavaScriptCore (default)

Uses Apple's JavaScriptCore engine via the `javascriptcore` crate. Full ES6+ support.

- Arrow functions, template literals, classes, destructuring, spread/rest
- `Math`, `JSON`, `Date`, `Promise`, `try/catch`
- `Array` — `map`, `filter`, `reduce`, `find`
- `String` — `toUpperCase`, `split`, `includes`, `substring`
- `Object` — `keys`, `values`

### Manual interpreter

A from-scratch JS interpreter written in Rust. Supports a subset of JavaScript:

- **Variables** — `let`, `const`, `var` declarations and reassignment
- **Types** — numbers, strings, booleans, `null`, `undefined`, objects, arrays
- **Operators** — arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`<`, `>`, `<=`, `>=`), equality (`==`, `===`, `!=`, `!==`), logical (`&&`, `||`, `!`)
- **Control flow** — `if/else`, `while`, `for`
- **Functions** — declarations, calls, return, closures
- **Data structures** — object literals with property access, array literals with indexing
- **`console.log`** — builtin output

## REPL

Run `ravel` with no arguments to start the interactive REPL. Features:

- Line history (up/down arrows)
- History persisted between sessions
- Use `--manual` flag for the manual backend

## Project structure

```
src/
  lexer/      — tokenizer
  parser/     — recursive descent parser + AST
  interpreter.rs — tree-walk interpreter
  builtins.rs    — builtin functions
  env.rs         — lexical environment
  value.rs       — runtime value types
  main.rs        — CLI entry point
```

## Tests

```bash
cargo test
```

96 passing tests across lexer, parser, interpreter, env, value, builtins, and integration.
