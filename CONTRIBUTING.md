# Contributing to fastlane-rfc2544

Thank you for your interest in contributing to fastlane-rfc2544!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/wwtraveler/fastlane-rfc2544.git`
3. Create a branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run tests: `cargo test --all`
6. Commit your changes: `git commit -am "Add feature X"`
7. Push to the branch: `git push origin feature/my-feature`
8. Open a Pull Request

## Development

### Building

```bash
cargo build --release
cargo build --target x86_64-unknown-linux-musl  # static binary
```

### Testing

```bash
cargo test --all                    # all unit tests
cargo test --test integration        # integration tests
./tests/e2e/run_e2e_tests.sh        # end-to-end tests
```

### Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common issues
- Add documentation comments (`///`) to public items

## Adding a New Test

1. Add the test type enum variant in `crates/fastlane-core/src/config.rs`
2. Create the test engine in `crates/fastlane-core/src/<test>.rs`
3. Add the binary run function in `crates/fastlane-core/src/<test>.rs`
4. Wire up the CLI argument in `crates/fastlane-cli/src/main.rs`
5. Add integration test in `crates/fastlane-core/tests/integration.rs`
6. Update the README with the new test

## Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Include tests for new code
- Update documentation as needed
- Add entries to CHANGELOG.md
- Squash commits for clarity

## Reporting Issues

Please include:
- fastlane version
- Interface type and speed
- Frame sizes tested
- Output format
- Steps to reproduce
