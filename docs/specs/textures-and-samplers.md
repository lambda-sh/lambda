---
title: "Textures and Samplers"
document_id: "texture-sampler-spec-2025-10-30"
status: "draft"
created: "2025-10-30T00:00:00Z"
last_updated: "2025-11-10T00:00:00Z"
version: "0.3.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "fc5eb52c74eb0835225959f941db8e991112b87d"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "textures", "samplers", "wgpu"]
---

# Textures and Samplers

Summary
- Introduces first-class 2D and 3D sampled textures and samplers with a
  builder-based application programming interface and platform abstraction.
- Rationale: Texture sampling is foundational for images, sprites, materials,
  volume data, and user interface elements; this specification establishes a
  portable surface to upload image data to the graphics processing unit (GPU)
  and sample in fragment shaders.

## Scope

### Goals

- Provide 2D and 3D color textures with initial data upload from the central
  processing unit (CPU) to the GPU.
- Provide samplers with common filtering and address modes (including W for
  3D).
- Integrate textures and samplers into bind group layouts and bind groups with
  explicit view dimensions.
- Expose a stable high-level application programming interface in `lambda-rs`
  backed by `wgpu` through `lambda-rs-platform`.
- Support sRGB and linear color formats appropriate for filtering.

### Non-Goals

- Storage textures, depth textures, cube maps, and 2D/3D array textures
  are out of scope for this revision and tracked under “Future Extensions”.
- Multisampled textures and render target (color attachment) workflows are
  out of scope for this revision and tracked under “Future Extensions”.
- Mipmap generation (automatic or offline); only level 0 is supported in this
  revision. See “Future Extensions”.
- Partial sub-rect updates; uploads are whole-image at creation time in this
  revision. See “Future Extensions”.

## Terminology

- Sampled texture: A read-only texture resource accessed in shaders with a
  separate sampler.
- Texture dimension: The physical dimensionality of the texture storage
  (`D2`, `D3`).
- View dimension: The dimensionality exposed to shader sampling
  (`D2`, `D3`).
- Texture view: A typed view of a texture used for binding; by default a full
  view is created for the specified dimension.
- Sampler: A state object defining filtering and address (wrap) behavior.
- Mipmap level (mip): A downscaled level of a texture. Level 0 is the base.
- sRGB: A gamma-encoded color format (`*Srgb`) with conversion to linear space
  on sampling.

## Architecture Overview

- Platform (`lambda-rs-platform`)
  - Wraps `wgpu::Texture`, `wgpu::TextureView`, and `wgpu::Sampler` with
    builders that perform validation and queue uploads with
    `wgpu::Queue::write_texture`.
  - Maps public `TextureFormat`, `TextureDimension`, `ViewDimension`, filter
    and address enums to `wgpu` types.
  - Owns raw `wgpu` handles; high-level layer interacts via opaque wrappers.

- High level (`lambda-rs`)
  - Public builders: `TextureBuilder` and `SamplerBuilder` in
    `lambda::render::texture`.
  - Bind group integration: extend existing builders to declare and bind
    sampled textures and samplers alongside uniform buffers. Layout entries
    specify view dimension explicitly or use 2D shorthands.
  - Keep `wgpu` details internal; expose stable enums and validation errors.

Data flow (creation and use):

```ascii
CPU pixels --> TextureBuilder (platform upload + validation)
          --> GPU Texture + default TextureView
SamplerBuilder --> GPU Sampler

BindGroupLayoutBuilder: uniform | sampled_texture | sampler
BindGroupBuilder:       buffer  | texture(view)   | sampler

Render pass: SetPipeline -> SetBindGroup -> Draw
```

## Design

### API Surface

- Platform layer (`lambda-rs-platform`, module `lambda_platform::wgpu::texture`)
  - Types: `Texture`, `Sampler` (own raw `wgpu` handles). A default
    full-range view is created and owned by `Texture` for binding.
  - Enums: `TextureFormat`, `TextureDimension` (`D2`, `D3`), `ViewDimension`
    (`D2`, `D3`), `FilterMode`, `AddressMode`.
  - Builders:
    - `TextureBuilder`
      - `new_2d(format: TextureFormat)` — convenience for `dimension = D2`.
      - `new_3d(format: TextureFormat)` — convenience for `dimension = D3`.
      - `with_size(width: u32, height: u32)` — required for 2D textures.
      - `with_size_3d(width: u32, height: u32, depth: u32)` — required for 3D.
      - `with_data(pixels: &[u8])` — upload full level 0; platform pads rows
        to satisfy `bytes_per_row` and sets `rows_per_image` for 3D.
      - `with_usage(binding: bool, copy_dst: bool)` — controls
        `TEXTURE_BINDING` and `COPY_DST` (default true for both).
      - `with_label(label: &str)`
      - `build(&mut RenderContext)` -> `Result<Texture, Error>`
    - `SamplerBuilder`
      - `new()`
      - Filtering: `nearest()`, `linear()`; shorthands `nearest_clamp()`,
        `linear_clamp()`.
      - Addressing: `with_address_mode_u(mode)`, `with_address_mode_v(mode)`,
        `with_address_mode_w(mode)`; default `ClampToEdge`.
      - Mip filtering and level-of-detail: `with_lod(min, max)`,
        `with_mip_filter(mode)` (default `Nearest`).
      - `with_label(label: &str)`
      - `build(&mut RenderContext)` -> `Sampler`

- High-level layer (`lambda-rs`, module `lambda::render::texture`)
  - Mirrors platform builders and enums; returns high-level `Texture` and
    `Sampler` wrappers with no `wgpu` exposure.
  - Adds convenience methods consistent with the repository style (for example,
    `SamplerBuilder::linear_clamp()`). Usage toggles MAY be exposed at the
    high level or fixed to stable defaults.

- Bind group integration (`lambda::render::bind`)
  - `BindGroupLayoutBuilder` additions:
    - `with_sampled_texture(binding: u32)` — 2D, filterable float; shorthand,
      default visibility Fragment.
    - `with_sampled_texture_dim(binding: u32, dim: ViewDimension, visibility: BindingVisibility)` —
      explicit dimension (`D2` or `D3`), float sample type, not multisampled.
    - `with_sampler(binding: u32)` — filtering sampler type; default visibility
      Fragment.
  - `BindGroupBuilder` additions:
    - `with_texture(binding: u32, texture: &Texture)` — uses the default view
      that matches the texture’s dimension.
    - `with_sampler(binding: u32, sampler: &Sampler)`.

### Behavior

- Texture creation
  - The builder constructs a texture with `mip_level_count = 1`,
    `sample_count = 1`, and `dimension` equal to `D2` or `D3`.
  - Usage includes `TEXTURE_BINDING` and `COPY_DST` by default.
  - When `with_data` is provided, the upload MUST cover the entire level 0.
    The platform layer pads rows to a `bytes_per_row` multiple of 256 bytes and
    sets `rows_per_image` for 3D before calling `Queue::write_texture`.
  - A default full-range `TextureView` is created and retained by the texture
    with `ViewDimension` matching the texture dimension.

- Sampler creation
  - Defaults: `FilterMode::Nearest` for min/mag/mip, `ClampToEdge` for all
    address modes, `lod_min_clamp = 0.0`, `lod_max_clamp = 32.0`.
  - Shorthands (`nearest_clamp`, `linear_clamp`) set min/mag filter and all
    address modes to `ClampToEdge`.

- Binding
  - `with_sampled_texture` declares a 2D filterable float texture binding at
    the specified index with Fragment visibility; shaders declare
    `texture_2d<f32>`.
  - `with_sampled_texture_dim` declares a texture binding with explicit view
    dimension and visibility; shaders declare `texture_2d<f32>` or
    `texture_3d<f32>`.
  - `with_sampler` declares a filtering sampler binding at the specified index
    with Fragment visibility; shaders declare `sampler` and combine with the
    texture in sampling calls.

### Validation and Errors

- Limits and dimensions
  - Width and height MUST be > 0 and ≤ the corresponding device limit for the
    chosen dimension.
  - 2D check: `≤ limits.max_texture_dimension_2d`.
  - 3D check: `≤ limits.max_texture_dimension_3d` for each axis.
  - Only textures with `mip_level_count = 1` are allowed in this revision.

- Format and sampling
  - `TextureFormat` MUST map to a filterable, color texture format in `wgpu`.
  - If a sampled texture is declared, the sample type is float with
    `filterable = true`; incompatible formats MUST be rejected at build time.

- Upload data
  - For 2D, `with_data` length MUST equal `width * height * bytes_per_pixel` of
    the chosen format for tightly packed rows.
  - For 3D, `with_data` length MUST equal
    `width * height * depth * bytes_per_pixel` for tightly packed input.
  - The platform layer performs padding to satisfy the 256-byte
    `bytes_per_row` requirement and sets `rows_per_image` appropriately;
    mismatched lengths or overflows MUST return an error before encoding.
  - If `with_data` is used, usage MUST include `COPY_DST`. An implementation
    MAY automatically add `COPY_DST` for build-time uploads to avoid errors.

- Bindings
  - `with_texture` and `with_sampler` MUST reference resources compatible with
    the corresponding layout entries, including view dimension; violations
    surface as validation errors from `wgpu` during bind group creation or
    render pass encoding.

## Constraints and Rules

- Alignment and layout
  - `bytes_per_row` MUST be a multiple of 256 bytes for `write_texture`.
  - The platform layer pads each source row to meet this requirement and sets
    `rows_per_image` for 3D writes.

- Supported formats (initial)
  - `Rgba8Unorm`, `Rgba8UnormSrgb`. Additional filterable color formats MAY be
    added in future revisions.

- Usage flags
  - Textures created for sampling MUST include `TEXTURE_BINDING`. When initial
    data is uploaded at creation, `COPY_DST` SHOULD be included.

## Performance Considerations

- Upload efficiency
  - Use a single `Queue::write_texture` per texture to minimize driver
    overhead. Rationale: Batching uploads reduces command submission costs.
- Large volume data
  - Prefer `copy_buffer_to_texture` for very large 3D uploads to reduce CPU
    staging and allow asynchronous transfers. Rationale: Improves throughput
    for multi-megabyte volumes.
- Filtering choice
  - Prefer `Linear` filtering for downscaled content; use `Nearest` for pixel
    art. Rationale: Matches visual expectations and avoids unintended blurring.
- Address modes
  - Use `ClampToEdge` for user interface and sprites; `Repeat` for tiled
    backgrounds. Rationale: Prevents sampling beyond image borders where not
    intended.

## Requirements Checklist

- Functionality
  - [x] Feature flags defined (if applicable) (N/A)
  - [x] 2D texture creation and upload
  - [x] 3D texture creation and upload
  - [x] Sampler creation (U, V, W addressing)
  - [x] Bind group layout and binding for texture + sampler (2D/3D)
- API Surface
  - [x] Public builders and enums in `lambda-rs`
  - [x] Platform wrappers in `lambda-rs-platform`
  - [x] Backwards compatibility assessed
- Validation and Errors
  - [ ] Dimension and limit checks (2D/3D)
  - [x] Format compatibility checks
  - [x] Data length and row padding/rows-per-image validation
- Performance
  - [x] Upload path reasoned and documented
  - [ ] Memory footprint characterized for common formats
- Documentation and Examples
  - [x] User-facing docs updated
  - [x] Minimal example rendering a textured quad (equivalent)
  - [x] Migration notes (if applicable) (N/A)

- Extensions (Planned)
  - [ ] Mipmapping: generation, `mip_level_count`, mip view selection
  - [ ] Texture arrays and cube maps: array/cube view dimensions and layout entries
  - [ ] Storage textures: read-write bindings and storage-capable formats
  - [ ] Render-target textures (color): `RENDER_ATTACHMENT` usage and MSAA resolve
  - [ ] Additional color formats: `R8Unorm`, `Rg8Unorm`, `Rgba16Float`, others
  - [ ] Compressed textures: BCn/ASTC/ETC via KTX2/BasisU
  - [ ] Anisotropic filtering and border color: anisotropy and `ClampToBorder`
  - [ ] Sub-rect updates and streaming: partial `write_texture`, buffer-to-texture
  - [ ] Alternate view formats: `view_formats` and view creation over subsets
  - [ ] Compare samplers (shadow sampling): comparison binding type and sampling
  - [ ] LOD bias and per-sample control: expose LOD bias and overrides

## Verification and Testing

- Unit Tests
  - Compute `bytes_per_row` padding and data size validation (2D/3D).
  - Compute `rows_per_image` for 3D uploads.
  - Format mapping from `TextureFormat` to `wgpu::TextureFormat`.
  - Commands: `cargo test -p lambda-rs-platform -- --nocapture`

- Integration Tests
  - Render a quad sampling a test checkerboard (2D); verify no validation
    errors and expected color histogram bounds.
  - Render a slice of a 3D texture by fixing W (for example, `uvw.z = 0.5`) in
    the fragment shader; verify sampling and addressing.
  - Commands: `cargo test --workspace`

- Manual Checks (if necessary)
  - Run example that draws a textured quad. Confirm correct sampling with
    `Nearest` and `Linear` and correct addressing with `ClampToEdge` and
    `Repeat`.

## Compatibility and Migration

- None. This is a new feature area. Future revisions MAY extend formats,
  dimensions, and render-target usage.

## Example Usage

Rust (2D high level)
```rust
use lambda::render::texture::{TextureBuilder, SamplerBuilder, TextureFormat};
use lambda::render::bind::{BindGroupLayoutBuilder, BindGroupBuilder, BindingVisibility};
use lambda::render::command::RenderCommand as RC;

let texture2d = TextureBuilder::new_2d(TextureFormat::Rgba8UnormSrgb)
  .with_size(512, 512)
  .with_data(&pixels)
  .with_label("albedo")
  .build(&mut render_context)?;

let sampler = SamplerBuilder::new()
  .linear_clamp()
  .with_label("albedo-sampler")
  .build(&mut render_context);

let layout2d = BindGroupLayoutBuilder::new()
  .with_uniform(0, BindingVisibility::VertexAndFragment)
  .with_sampled_texture(1) // 2D shorthand
  .with_sampler(2)
  .build(&mut render_context);

let group2d = BindGroupBuilder::new()
  .with_layout(&layout2d)
  .with_uniform(0, &uniform_buffer)
  .with_texture(1, &texture2d)
  .with_sampler(2, &sampler)
  .build(&mut render_context);

RC::SetBindGroup { set: 0, group: group_id, dynamic_offsets: vec![] };
```

WGSL snippet (2D)
```wgsl
@group(0) @binding(1) var texture_color: texture_2d<f32>;
@group(0) @binding(2) var sampler_color: sampler;

@fragment
fn fs_main(in_uv: vec2<f32>) -> @location(0) vec4<f32> {
  let color = textureSample(texture_color, sampler_color, in_uv);
  return color;
}
```

Rust (3D high level)
```rust
use lambda::render::texture::{TextureBuilder, TextureFormat};
use lambda::render::bind::{BindGroupLayoutBuilder, BindGroupBuilder};
use lambda::render::texture::ViewDimension;
use lambda::render::bind::BindingVisibility;

let texture3d = TextureBuilder::new_3d(TextureFormat::Rgba8Unorm)
  .with_size_3d(128, 128, 64)
  .with_data(&voxels)
  .with_label("volume")
  .build(&mut render_context)?;

let layout3d = BindGroupLayoutBuilder::new()
  .with_sampled_texture_dim(1, ViewDimension::D3, BindingVisibility::Fragment)
  .with_sampler(2)
  .build(&mut render_context);

let group3d = BindGroupBuilder::new()
  .with_layout(&layout3d)
  .with_texture(1, &texture3d)
  .with_sampler(2, &sampler)
  .build(&mut render_context);
```

WGSL snippet (3D)
```wgsl
@group(0) @binding(1) var volume_tex: texture_3d<f32>;
@group(0) @binding(2) var volume_samp: sampler;

@fragment
fn fs_main(in_uv: vec2<f32>) -> @location(0) vec4<f32> {
  // Sample a middle slice at z = 0.5
  let uvw = vec3<f32>(in_uv, 0.5);
  return textureSample(volume_tex, volume_samp, uvw);
}
```

## Changelog

- 2025-11-10 (v0.3.1) — Merge “Future Extensions” into the Requirements
  Checklist and mark implemented status; metadata updated.
- 2025-11-09 (v0.3.0) — Clarify layout visibility parameters; make sampler
  build infallible; correct `BindingVisibility` usage in examples;
  add “Future Extensions” with planned texture features; metadata updated.
- 2025-10-30 (v0.2.0) — Add 3D textures, explicit dimensions in layout and
  builders, W address mode, validation and examples updated.
- 2025-10-30 (v0.1.0) — Initial draft.

## Future Extensions

Moved into the Requirements Checklist under “Extensions (Planned)”.
