# AI Agent Guidelines

## Development Workflow

- **Implementation:** Build features using Rust and rquickjs.
- **Unit Testing:** Write unit tests for every new feature in `src/`.
- **Examples:** Create a new `examples/*.js` file for any new functionality.
- **Snapshot Testing:** Generate/update snapshot tests whenever a new example is added.
- **Integration:** Update `tests/integration_test.rs` to reflect system-wide changes.
- **Documentation:** Always update `README` to match the current status after tests pass.

## Testing Standards

- Ensure all tests pass via `cargo test`.
- Verify JavaScript behavior by capturing and asserting Rust-side output.
- Validate that sandboxing (e.g., fs restrictions) remains intact after modifications.

## Constraints

- No feature additions without accompanying tests.
- No drift between implementation and `README`.
- Prioritize Safe Rust when interacting with the rquickjs engine.
