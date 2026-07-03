## Summary

Add an opt-in 2D physics world API (`PhysicsWorld2D`) to `lambda-rs`, backed by
Rapier (`rapier2d =0.32.0`) via `lambda-rs-platform`, and provide a minimal demo
and tutorial that render a falling 2D quad while stepping an empty physics
world.

## Related Issues

- Relates to: (none linked)

## Changes

- Add `lambda::physics::PhysicsWorld2D` and `PhysicsWorld2DBuilder` behind the
  `physics-2d` feature, including validation and fixed-timestep stepping.
- Add `lambda_platform::physics::rapier2d` backend wrapper and wire it behind
  `lambda-rs-platform/physics-2d`.
- Add unit tests for builder validation and stepping an empty world.
- Add a dedicated physics demos crate (`demos/physics`) and a minimal falling
  quad demo (`physics_falling_quad`) that defaults to `physics-2d` enabled.
- Add a specification for the 2D physics world under `docs/specs/physics/`.
- Update `docs/features.md` to document `physics-2d` feature flags and their
  dependency relationships.
- Add a tutorial documenting how to build the falling quad demo from scratch.

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [x] Feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [x] Documentation (updates to docs, specs, tutorials, or comments)
- [ ] Refactor (code change that neither fixes a bug nor adds a feature)
- [ ] Performance (change that improves performance)
- [x] Test (adding or updating tests)
- [ ] Build/CI (changes to build process or CI configuration)

## Affected Crates

- [x] `lambda-rs`
- [x] `lambda-rs-platform`
- [ ] `lambda-rs-args`
- [ ] `lambda-rs-logging`
- [x] Other: `demos/physics`, docs

## Checklist

- [ ] Code follows the repository style guidelines (`cargo +nightly fmt --all`)
- [ ] Code passes clippy (`cargo clippy --workspace --all-targets -- -D warnings`)
- [ ] Tests pass (`cargo test --workspace`)
- [x] New code includes appropriate documentation
- [x] Public API changes are documented
- [ ] Breaking changes are noted in this PR description

## Testing

**Commands run:**

```bash
# Suggested minimum coverage for this PR:
cargo test -p lambda-rs --features physics-2d

# Demo:
cargo run -p lambda-demos-physics --bin physics_falling_quad
```

**Manual verification steps (if applicable):**

1. Run `cargo run -p lambda-demos-physics --bin physics_falling_quad`.
2. Confirm a 2D quad falls under gravity and remains visible while the physics
   world steps each fixed tick.

## Screenshots/Recordings

- (optional) attach a short clip of the falling quad demo.

## Platform Testing

- [x] macOS
- [ ] Windows
- [ ] Linux

## Additional Notes

- Public physics API intentionally remains minimal: no application-facing rigid
  bodies, colliders, or collision queries are introduced in this work item.
- Physics is feature-gated (`physics-2d`) to avoid pulling physics dependencies
  into minimal builds.
