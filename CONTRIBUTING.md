# Contributing to MeshRelay

## Development

The relay daemon is written in Rust (Axum + SQLite). The web reader is a single HTML file with no dependencies.

```bash
cd relay-daemon
cp config.example.toml config.toml
cargo run --release
```

## Coding Standards

- Rust: Run `cargo fmt` and `cargo clippy` before submitting. Warnings are treated as errors in CI.
- Web reader: Vanilla HTML + CSS + JavaScript. No frameworks.
- Python bridges: PEP 8 style.

## Pull Requests

- One focused change per PR
- Include a description of what the change does and why
- If adding a feature, explain the use case
- CI must pass (cargo check, clippy, tests, shell script validation)

## License

MIT. By contributing, you agree to license your changes under the MIT license.
