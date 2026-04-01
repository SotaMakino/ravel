# TODO List - Ravel (Toy JS Runtime)

## High Priority

- [ ] **Add interpreter tests** — No tests exist for `interpreter.rs`, `env.rs`, `builtins.rs`, or `value.rs`
- [ ] **Add integration tests** — End-to-end file execution tests
- [ ] **Implement `break` and `continue`** — Loops exist but lack control statements
- [ ] **Enforce `const` immutability** — `const` variables can currently be reassigned
- [ ] **Fix member assignment** — `obj.prop = value` uses a placeholder name and doesn't work correctly
- [ ] **Implement ternary operator** — `?` and `:` tokens are lexed but not parsed

## Medium Priority

- [ ] **Add more built-in functions** — `print`, `parseInt`, `parseFloat`, `Math`, `Array` methods, `Object` methods, `String` methods
- [ ] **Implement `try/catch/throw`** — No exception handling
- [ ] **Add `switch` statement**
- [ ] **Add `do...while` loop**
- [ ] **Implement bitwise operators** — `&`, `|`, `^`, `~`, `<<`, `>>`, `>>>`
- [ ] **Add `typeof`, `instanceof`, `in`, `delete` operators**
- [ ] **Support arrow functions**
- [ ] **Support template literals**

## Low Priority

- [ ] **Add `class` syntax** — No class or prototype support
- [ ] **Support `this` keyword**
- [ ] **Support spread/rest syntax** — `...`
- [ ] **Write README/documentation**

## Infrastructure

- [ ] **Initial git commit** — Repository has no commits yet
