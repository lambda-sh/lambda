Temporary implementation plan for `SoundBuffer` audio file loading (WAV + OGG
Vorbis) using `symphonia =0.5.5` in `lambda-rs-platform`.

Each step is intended to be independently testable and committable.

1) Refactor `lambda-rs` audio module layout (no behavior change)
- Changes
  - Move `crates/lambda-rs/src/audio.rs` → `crates/lambda-rs/src/audio/mod.rs`.
  - Update `crates/lambda-rs/src/lib.rs` module path accordingly.
  - Keep compilation gated by `audio-output-device` for now.
- Validation
  - `cargo build -p lambda-rs --features audio-output-device`
- Commit message
  - `[refactor] move lambda-rs audio module to mod.rs.`

2) Add `SoundBuffer` feature flags + module gating in `lambda-rs` (no decoding yet)
- Changes
  - Update `crates/lambda-rs/Cargo.toml`:
    - Add granular features: `audio-sound-buffer-wav`,
      `audio-sound-buffer-vorbis`.
    - Add umbrella feature: `audio-sound-buffer`.
    - Update umbrella `audio` to include `audio-output-device` and
      `audio-sound-buffer`.
  - Update `crates/lambda-rs/src/lib.rs`:
    - Compile `pub mod audio` when
      `any(feature = "audio-output-device", feature = "audio-sound-buffer")`.
- Validation
  - `cargo build -p lambda-rs --no-default-features --features audio-sound-buffer-wav`
- Commit message
  - `[add] lambda-rs sound buffer feature flags.`

3) Add platform decode features + `symphonia =0.5.5` dependency plumbing
- Changes
  - Update `crates/lambda-rs-platform/Cargo.toml`:
    - Add optional dependency:
      - `symphonia = "=0.5.5"`, `default-features = false`
      - Enable only required Symphonia features for WAV + OGG Vorbis (expected:
        `ogg`, `vorbis`, `wav`, `pcm`).
    - Add new granular features:
      - `audio-decode-wav`
      - `audio-decode-vorbis`
    - Update platform umbrella `audio` to include:
      - `audio-device`
      - `audio-decode-wav`
      - `audio-decode-vorbis`
  - Update `crates/lambda-rs-platform/src/lib.rs`:
    - Ensure `pub mod audio` compiles when any audio feature is enabled
      (device or decode).
- Validation
  - `cargo build -p lambda-rs-platform --no-default-features --features audio-decode-wav`
- Commit message
  - `[add] platform audio decode feature plumbing.`

4) Implement platform wrapper scaffold: `lambda_platform::audio::symphonia`
- Changes
  - Add `crates/lambda-rs-platform/src/audio/symphonia/mod.rs`:
    - Define a backend-agnostic `DecodedAudio` struct.
    - Define a vendor-free `AudioDecodeError` error type.
    - Add byte-based entry points (stubs are acceptable initially):
      - `decode_wav_bytes(bytes: &[u8]) -> Result<DecodedAudio, AudioDecodeError>`
      - `decode_ogg_vorbis_bytes(bytes: &[u8]) -> Result<DecodedAudio, AudioDecodeError>`
    - Perform minimal format hinting / probing and return actionable
      unsupported/invalid errors without exposing Symphonia error types.
- Validation
  - `cargo test -p lambda-rs-platform --no-default-features --features audio-decode-wav`
- Commit message
  - `[add] symphonia platform wrapper scaffold.`

5) Implement actual decoding in platform wrapper (WAV + OGG Vorbis)
- Changes
  - Complete Symphonia-based decode paths to fully decode into interleaved
    `Vec<f32>`.
  - Validate and reject:
    - `channels == 0` or `sample_rate == 0`
    - channel counts outside required scope (initial: 1–2)
    - Ogg streams that are not Vorbis for the Vorbis entry point
  - Map all failures into `AudioDecodeError` without leaking vendor types.
- Validation
  - `cargo test -p lambda-rs-platform --no-default-features --features audio-decode-wav,audio-decode-vorbis`
- Commit message
  - `[add] symphonia decoding for wav and ogg vorbis.`

6) Add `SoundBuffer` public API in `lambda-rs` + extend `AudioError`
- Changes
  - Add `crates/lambda-rs/src/audio/buffer.rs`.
  - Update `crates/lambda-rs/src/audio/mod.rs`:
    - Re-export `SoundBuffer`.
    - Extend `AudioError` with explicit I/O + decode variants used by sound
      buffer loading.
    - Keep error messages actionable and backend-agnostic.
  - Implement:
    - `SoundBuffer::from_*_file` reads full file bytes then decodes.
    - `SoundBuffer::from_*_bytes` decodes without filesystem access.
    - `duration_seconds` computed from frames/channels/sample_rate.
  - Map platform decode errors into `lambda::audio::AudioError`.
- Validation
  - `cargo build -p lambda-rs --no-default-features --features audio-sound-buffer`
- Commit message
  - `[add] SoundBuffer type and loading API.`

7) Add small test assets under `crates/lambda-rs-platform/assets/audio/`
- Changes
  - Add `crates/lambda-rs-platform/assets/audio/` fixtures:
    - WAV: mono/stereo 16-bit PCM, mono 24-bit PCM, stereo 32-bit float
    - OGG: mono + stereo Vorbis
- Validation
  - `rg --files crates/lambda-rs-platform/assets/audio`
- Commit message
  - `[add] audio decode test fixtures.`

8) Add unit tests for `SoundBuffer` using fixtures
- Changes
  - Add tests in `crates/lambda-rs/src/audio/buffer.rs` (or a dedicated test
    module) that use `include_bytes!` fixtures.
  - Assert:
    - `channels`, `sample_rate`, and `duration_seconds` math
  - Add negative tests for unsupported/corrupt inputs.
- Validation
  - `cargo test -p lambda-rs --no-default-features --features audio-sound-buffer -- --nocapture`
- Commit message
  - `[add] unit tests for SoundBuffer decoding.`

9) Add example: `sound_buffer_load`
- Changes
  - Add `demos/audio/src/bin/sound_buffer.rs`:
    - Load a path from CLI args.
    - Select WAV vs OGG by extension (or flags).
    - Print channels/sample_rate/duration.
- Validation
  - `cargo run -p lambda-demos-audio --bin sound_buffer -- <path>`
- Commit message
  - `[add] SoundBuffer loading example.`

10) Docs + workspace-wide checks
- Changes
  - Update `docs/features.md` for new audio features (lambda-rs + platform),
    including default state and expected runtime cost.
  - Keep `docs/specs/audio-file-loading.md` consistent with any final symbol
    names and feature wiring decisions.
- Validation
  - `cargo +nightly fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- Commit message
  - `[docs] document audio sound buffer features.`
