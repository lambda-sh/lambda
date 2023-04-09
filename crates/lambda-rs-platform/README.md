# lambda-rs-platform
![lambda-rs](https://img.shields.io/crates/d/lambda-rs-platform)
![lambda-rs](https://img.shields.io/crates/v/lambda-rs-platform)

Platform implementations for lambda-rs. This crate is not intended to be used directly and guarantees no stability across versions.

## Platforms
The following platforms are currently supported:
* Rendering & Compute support
  * Windows
    * Vulkan
    * DirectX 11
    * DirectX 12
  * Linux
    * Vulkan
    * OpenGL
  * MacOS
    * Metal
    * Vulkan
* Window support
  * winit
* UI support
  * egui (via winit)
