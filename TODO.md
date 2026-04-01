# TODO List - Ravel (Toy JS Runtime)

## What Works

### JavaScriptCore Backend (`--jsc`)
- **Full ES6+** — Arrow functions, template literals, classes, destructuring, spread, promises, try/catch
- **Standard library** — `Math`, `JSON`, `Date`, `Object.keys/values`, `Array.map/filter/reduce/find`, `String.toUpperCase/split/includes/substring`
- **`console.log`** — Custom Rust callback via `function_callback`

### Tests
- **96 passing** — 12 lexer, 10 parser, 34 interpreter, 7 env, 2 builtins, 17 value, 14 integration

## TODO

### Builtins / Standard Library

- [ ] **More builtins** — `print`, `parseInt`, `parseFloat`, `Math`, `Array` methods, `Object` methods, `String` methods
- [ ] **Standard library** — `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`, `fetch`
- [ ] **Event loop** — Task queue, microtask queue, `queueMicrotask`

### Documentation

- [ ] **Write README** — Project overview, usage, features
