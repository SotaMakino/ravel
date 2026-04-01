# TODO List - Ravel (Toy JS Runtime)

## What Works

### Manual Backend (Toy Interpreter)
- **Variables** — `let`, `const`, `var` declarations and reassignment
- **Data types** — Numbers, strings, booleans, null, undefined, objects, arrays
- **Operators** — Arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`), logical (`&&`, `||`, `!`)
- **Control flow** — `if/else`, `while`, `for` (C-style)
- **Functions** — `function` declarations, parameters, return, closures
- **Property access** — `obj.prop`, `arr[0]`, `obj["key"]`
- **Builtins** — `console.log`
- **CLI** — File execution (`ravel file.js`) and REPL mode
- **Comments** — Line (`//`) and block (`/* */`)

### JavaScriptCore Backend (`--jsc`)
- **Full ES6+** — Arrow functions, template literals, classes, destructuring, spread, promises, try/catch
- **Standard library** — `Math`, `JSON`, `Date`, `Object.keys/values`, `Array.map/filter/reduce/find`, `String.toUpperCase/split/includes/substring`
- **`console.log`** — Custom Rust callback via `function_callback`

### Tests
- **82 passing** — 12 lexer, 10 parser, 34 interpreter, 7 env, 2 builtins, 17 value

## TODO

### Core Fixes

- [ ] **Fix member assignment** — `obj.prop = value` uses a placeholder and doesn't work
- [ ] **Enforce `const` immutability** — `const` variables can be reassigned
- [ ] **Add integration tests** — End-to-end file execution tests

### Language Features (Manual Backend)

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

### Builtins (Manual Backend)

- [ ] **More builtins** — `print`, `parseInt`, `parseFloat`, `Math`, `Array` methods, `Object` methods, `String` methods

### Documentation

- [ ] **Write README** — Project overview, usage, features
