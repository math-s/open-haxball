# Claude Rules

## Rust Server (packages/server-rust)

After modifying any Rust file in `packages/server-rust/`, run the tests:

```bash
cd packages/server-rust && cargo test
```

### Quick test commands

```bash
# All tests
cargo test

# Specific module
cargo test physics::collision
cargo test game::tests

# Integration tests only
cargo test --test game_integration
```
