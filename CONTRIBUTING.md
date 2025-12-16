# Contributing to Lambda

Thank you for your interest in contributing to Lambda. This document provides
guidelines for contributing to the project.

## Getting Started

1. Fork the repository and clone it locally.
2. Run `scripts/setup.sh` to install git hooks and git-lfs.
3. Build the workspace: `cargo build --workspace`
4. Run tests: `cargo test --workspace`

## Repository Guidelines

Refer to [`AGENTS.md`](./AGENTS.md) for detailed information about:

- Project structure and module organization
- Architecture and design principles
- Coding practices and style conventions
- Testing guidelines
- Documentation standards

## Code Style

Before submitting changes:

```bash
# Format code (requires nightly toolchain)
cargo +nightly fmt --all

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace
```

## Commit Messages

Follow the `[scope] message` convention:

- `[add]` — New feature or file
- `[fix]` — Bug fix
- `[refactor]` — Code restructuring without behavior change
- `[docs]` — Documentation changes
- `[test]` — Test additions or modifications
- `[build]` — Build system or CI changes

Examples:

```
[add] input action mapping system to lambda-rs.
[fix] mouse press events missing cursor coordinates.
[docs] add uniform buffer tutorial.
[refactor] extract render command encoding to separate module.
```

## Pull Requests

1. Create a branch from `main` with a descriptive name.
2. Make focused, narrowly scoped commits.
3. Ensure all commits build independently.
4. Fill out the PR template completely.
5. Link related issues using "Closes #123" or "Relates to #456".
6. Include screenshots or recordings for visual changes.
7. List manual verification steps if automated testing is insufficient.

## Issues

Use the appropriate issue template:

- **Feature Request**: Propose new functionality or enhancements.
- **Bug Report**: Report incorrect behavior or defects.
- **Documentation**: Report missing or unclear documentation.

Provide sufficient detail for maintainers to evaluate and reproduce issues.

## Documentation

When adding or modifying features:

- Update rustdoc comments for public APIs.
- Add or update examples in `crates/lambda-rs/examples/`.
- Create or update specifications in `docs/specs/` for significant changes.
- Follow the documentation tone and style guidelines in `AGENTS.md`.

## Questions and Discussions

For general questions or ideas, use
[GitHub Discussions](https://github.com/lambda-sh/lambda/discussions).

## License

By contributing, you agree that your contributions will be licensed under the
same license as the project (see [`LICENSE`](./LICENSE)).
