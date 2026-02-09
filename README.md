<p align="center" style="text-align:center;">
<img src="https://media.githubusercontent.com/media/lambda-sh/lambda/main/crates/lambda-rs/assets/logos/lambda_header.png" />
</p>

[![Cross Platform builds & tests](https://github.com/lambda-sh/lambda/actions/workflows/compile_lambda_rs.yml/badge.svg)](https://github.com/lambda-sh/lambda/actions/workflows/compile_lambda_rs.yml)
[![Release](https://github.com/lambda-sh/lambda/actions/workflows/release.yml/badge.svg)](https://github.com/lambda-sh/lambda/actions/workflows/release.yml)
![lambda-rs](https://img.shields.io/crates/d/lambda-rs)
![lambda-rs](https://img.shields.io/crates/v/lambda-rs)

## Table of contents

1. [Description](#description)
1. [API Documentation](#documentation)
1. [Building](#building)
    1. [From crates.io](#crates)
    1. [From source](#source)
        1. [External dependencies](#ext_deps)
        1. [Optional dependencies](#opt_deps)
        1. [Linux, Macos, Windows](#bash)
1. [Getting started](#get_started)
1. [Tutorials](#tutorials)
1. [Examples](#examples)
1. [Planned additions](#plans)
1. [Releases & Publishing](#publishing)
1. [How to contribute](#contribute)
1. [Resources](#resources)

## Description <a name="description"></a>

Lambda is a framework for developing cross platform applications and workloads using Rust.

Lambda aims to enable developers to create highly performant, portable, and
minimal desktop applications by providing a platform agnostic API for all of the features that any application or workload might need.

Lambda :

* Desktop applications
* Productivity tools
* Data visualizations
* Physical system simulations
* Games

Over the last couple of years, the prominence of the web has replaced
traditional desktop applications. While this has lead to a golden age for
developing UI/UX for applications across all platforms, it has come at the
cost of degraded performance & resource consumption.

Lambda's goal isn't to replace electron, webview, or other similar web based
desktop frameworks, however; it is to instead create a cross platform ecosystem
for desktop applications with performance and resource consumption at the
forefront of it's priorities without sacrificing good UI/UX. Lambda may offer
lightweight HTML/CSS based rendering in the future but we're primarily focused
on implementing a Rust native UI framework built on top of our rendering engine.

## Documentation <a name="documentation"></a>

* [lambda-rs API documentation](https://docs.rs/lambda-rs/2023.1.29/lambda/)

## Installation <a name="building"></a>

### From crates.io <a name="crates"></a>

lambda is now available on [crates.io](https://crates.io/crates/lambda-rs)
and can be added to your project by adding the following to your
`Cargo.toml` file:

```toml
[dependencies]
lambda-rs = "2023.1.30"
```

or from the CLI:

```bash
cargo add lambda-rs
```

### From source <a name="source"></a>

#### Required external dependencies <a name="ext_deps"></a>

* All platforms
  * `git` is needed to clone the project and manage it's dependencies.
  * `git-lfs` is needed for asset files.
  * `rust >= 1.60` is needed for compiling lambda and all of it's crates.
  * `pre-commit` is used for development git commit hooks and any changes that do not pass the pre-commit checks will not be accepted.

#### Rendering API support

These are the Rendering APIs that are supported on each platform and must be
installed manually. More information on how to choose which backend lambda
uses on each platform is provided further below.

* Windows
  * `OpenGL`
  * `Vulkan`
  * `DirectX11`
  * `DirectX12`
* Linux
  * `OpenGL`
  * `Vulkan`
* MacOS
  * `Metal`
  * `Vulkan`

#### Linux (bash), MacOS (bash), Windows (git-bash) <a name="bash"></a>

If planning to develop for lambda, you must run the setup script provided by repository like so:

```bash
./scripts/setup.sh
```

This will initialize pre commit checks for development use and setup git-lfs for asset management.

In order to validate that lambda successfully compiles, you can build the library by performing a build with cargo.

```bash
cargo build --lib
```

If this works successfully, then lambda is ready to work on your system!

## Getting started <a name="get_started"></a>

Coming soon.

## Tutorials <a name="tutorials"></a>

Start with the tutorials to build features step by step:

* Tutorials index: [docs/tutorials/](./docs/tutorials/)
* Uniform Buffers: Build a Spinning Triangle: [docs/tutorials/rendering/resources/uniform-buffers.md](./docs/tutorials/rendering/resources/uniform-buffers.md)

## Examples <a name="examples"></a>

Browse runnable demos and example sources:

* Demo crates (recommended): [demos/](./demos/)
* Minimal rustdoc examples: [crates/lambda-rs/examples/](./crates/lambda-rs/examples/)
* Logging examples: [crates/lambda-rs-logging/examples/](./crates/lambda-rs-logging/examples/)
* Argument parsing examples: [crates/lambda-rs-args/examples/](./crates/lambda-rs-args/examples/)

### Minimal

A minimal demo of an application with a working window using lambda.

```bash
cargo run -p lambda-demos-minimal --bin minimal
```

### Immediates

An example of using shaders with immediates (per-draw data) to render a 3D image.

```bash
cargo run -p lambda-demos-render --bin immediates
```

#### Notes

* On windows, you need to run this example with
`--features lambda-rs/with-vulkan` as the shader used in the example does not work
in either dx11 or dx12.

### Triangle

An example using shaders to render a single triangle.

```bash
cargo run -p lambda-demos-render --bin triangle
```

### Triangles

An example using shaders to render multiple triangles and keyboard input to move one of the triangles on screen.

```bash
cargo run -p lambda-demos-render --bin triangles
```

## Plans <a name="plans"></a>

* ### Architecture support

  * [x] x86
  * [x] arm64

* ### Operating system support

  * [x] MacOS
  * [x] Linux
  * [x] Windows 10/11
  * [ ] Xbox Series S|X (Long term goal)
  * [ ] iOS (Long term goal)
  * [ ] Android (Long term goal)

* ### Rendering API support

  * [x] OpenGL
  * [x] Vulkan
  * [x] Metal
  * [x] DirectX11
  * [x] DirectX12

* ### Packages

  * [x] (WIP) [lambda-rs-args](./crates/lambda-rs-args/README.md) -- Command line argument parsing.
  * [x] (WIP) [lambda-rs-platform](./crates/lambda-rs-platform/README.md) -- Dependency wrappers & platform support.
  * [x] [lambda-rs-logging](./crates/lambda-rs-logging/README.md) -- Lightweight Logging API for lambda-rs packages.
  * [x] (WIP) [lambda-rs](./crates/lambda-rs/README.md) -- The public Lambda API.

* ### Examples

  * [x] Minimal -- A minimal example of an application with a working window
  using lambda.
  * [x] Immediates -- An example of using shaders with immediate data to
  render a 3D image.
  * [x] Triangle -- An example using shaders to render a single triangle.
  * [x] Triangles -- An example using shaders to render multiple triangles and keyboard input to move one of the triangles on screen.

* ### Tools

  * [x] obj-loader -- (WIP) Loads .obj files into lambda. Meshes need to be triangulated in order for it to render at the moment.
  * [ ] platform-info -- Utility for viewing information about the current platform.

* ### CI/CD

  * [x] Github action pipelines for building lambda on all platforms.
  * [ ] Github action pipelines for creating downloadable builds from releases.
  * [ ] Tests & benchmarking.
    * [ ] Unit tests.
  * [ ] Nightly builds.

## Releases & Publishing <a name="publishing"></a>

For cutting releases, publishing crates to crates.io, and attaching
multi-platform artifacts to GitHub Releases, see:

* docs/publishing.md

## How to contribute <a name="contribute"></a>

Fork the current repository and then make the changes that you'd like to
said fork. Stable releases will happen within the main branch requiring that
additions to be made off of `dev` which is the nightly build branch for lambda.
Upon adding features, fixing bugs, or whatever modifications you've made to the
project, issue a pull request into the latest `dev` branch containing the
changes that you've made and I will evaluate them before taking further action.
This process may take anywhere from 3-7 days depending on the scope of the
changes made, my schedule, and any other variable factors. They must also pass
all of the build pipelines are configured to run at merge.

## Resources <a name="resources"></a>

[The Cherno's playlist for making a game engine](https://www.youtube.com/playlist?list=PLlrATfBNZ98dC-V-N3m0Go4deliWHPFwT)

[Creator of Logo](https://github.com/RinniSwift)
