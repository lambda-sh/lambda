# lambda-audio

`lambda-audio` is a small CLI tool for inspecting and playing sound files using
the `lambda` audio APIs.

Scope
- Load sound files into `lambda::audio::SoundBuffer`.
- Print decoded metadata (sample rate, channels, duration).
- Render a small ASCII waveform preview.
- Play the decoded audio via the default output device.

## Usage

From the repository root:

```bash
cargo run -p lambda-audio-tool -- <command> [path]
```

Commands
- `info <path>`: decode and print metadata.
- `view <path>`: decode, print metadata, and show an ASCII waveform preview.
- `play <path>`: decode, print metadata, and play through the default output
  device.
- `list-devices`: list available output devices (platform-dependent).

Supported file types
- `.wav`
- `.ogg` / `.oga` (Vorbis)

## Examples (slash fixture)

The repository includes an OGG Vorbis fixture at:
`crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg`.

### `info`

```bash
cargo run -p lambda-audio-tool -- info \
  crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg
```

Example output:

```text
path: crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg
sample_rate: 48000
channels: 2
frames: 53824
samples: 107648
duration_seconds: 1.121
```

### `view`

```bash
cargo run -p lambda-audio-tool -- view \
  crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg
```

Example output:

```text
path: crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg
sample_rate: 48000
channels: 2
frames: 53824
samples: 107648
duration_seconds: 1.121

         ###
        #####
        ######
        ######
        ######
       ########   #
       #############
      #################
     ###########################
```

### `play`

```bash
cargo run -p lambda-audio-tool -- play \
  crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg
```

Example output (audio plays, then the process exits):

```text
path: crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg
sample_rate: 48000
channels: 2
frames: 53824
samples: 107648
duration_seconds: 1.121
```

### `list-devices`

```bash
cargo run -p lambda-audio-tool -- list-devices
```

Example output (varies by machine and environment):

```text
no output devices found
```
