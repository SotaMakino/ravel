# TODO List - Ravel (Toy JS Runtime)

## What Works

- **Variables** — `let`, `const`, `var` declarations and reassignment
- **Data types** — Numbers, strings, booleans, null, undefined, objects, arrays
- **Operators** — Arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`), logical (`&&`, `||`, `!`)
- **Control flow** — `if/else`, `while`, `for` (C-style)
- **Functions** — `function` declarations, parameters, return, closures
- **Property access** — `obj.prop`, `arr[0]`, `obj["key"]`
- **Builtins** — `console.log`
- **CLI** — File execution (`ravel file.js`) and REPL mode
- **Comments** — Line (`//`) and block (`/* */`)
- **Tests** — 22 passing (12 lexer, 10 parser)

## TODO

### Core Fixes

- [ ] **Fix member assignment** — `obj.prop = value` uses a placeholder and doesn't work
- [ ] **Enforce `const` immutability** — `const` variables can be reassigned
- [ ] **Add interpreter tests** — No tests for `interpreter.rs`, `env.rs`, `builtins.rs`, `value.rs`
- [ ] **Add integration tests** — End-to-end file execution tests

### Language Features

- [ ] **`break` and `continue`** — Loop control statements
- [ ] **Ternary operator** — `cond ? a : b` (tokens already lexed)
- [ ] **Bitwise operators** — `&`, `|`, `^`, `~`, `<<`, `>>`, `>>>`
- [ ] **`typeof`, `instanceof`, `in`, `delete`**
- [ ] **Arrow functions** — `() => {}`
- [ ] **Template literals** — `` `hello ${name}` ``
- [ ] **Spread/rest syntax** — `...`
- [ ] **`this` keyword**
- [ ] **`class` syntax** — Classes and prototype chain
- [ ] **`switch` statement**
- [ ] **`do...while` loop**
- [ ] **`try/catch/throw`** — Exception handling

### Builtins

- [ ] **More builtins** — `print`, `parseInt`, `parseFloat`, `Math`, `Array` methods, `Object` methods, `String` methods

### Documentation

- [ ] **Write README** — Project overview, usage, features
