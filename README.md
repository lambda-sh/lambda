<p align="center" style="text-align:center;">
<img src="https://media.githubusercontent.com/media/lambda-sh/lambda/main/lambda/assets/logos/lambda_header.png" />
</p>

[![compile & test lambda-rs](https://github.com/lambda-sh/lambda/actions/workflows/compile_lambda_rs.yml/badge.svg)](https://github.com/lambda-sh/lambda/actions/workflows/compile_lambda_rs.yml)



## Table of contents
1. [Description](#description)
2. [API Documentation](#documentation)
3. [Building](#building)
    1. [From source](#source)
        1. [External dependencies](#ext_deps)
        2. [Optional depedencies](#opt_deps)
        3. [Linux, Macos, Windows](#bash)
5. [Getting started](#get_started)
6. [Examples](#examples)
7. [Planned additions](#plans)
8. [How to contribute](#contribute)
9. [Resources](#resources)
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
Documentation to be added soon.

## Building <a name="building"></a>

### From source <a name="source"></a>
Currently, building from source is the only method to obtain a copy of Lambda.

#### Required external dependencies <a name="ext_deps"></a>
* All platforms
	* `cmake >= 3.20.0` is needed to build shaderc from source.
	* `ninja` is needed to build shaderc from source.
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

## Examples <a name="examples"></a>
### Minimal
A minimal example of an application with a working window using lambda.
```rust
cargo run --example minimal
```

### Push Constants
An example of using shaders with push constants to render a 3D image.
```rust
cargo run --example push_constants
```

### Triangle
An example using shaders to render a single triangle.
```rust
cargo run --example triangle
```

### Triangles
An example using shaders to render multiple triangles and keyboard input to move one of the triangles on screen.
```rust
cargo run --example triangles
```

## Plans <a name="plans"></a>
- ### Architecture support
	- [x] x86
	- [x] arm64
- ### Operating system support
	- [x] MacOS
	- [x] Linux
	- [x] Windows 10/11
	- [ ] Xbox Series S|X (Long term goal)
	- [ ] iOS (Long term goal)
	- [ ] Android (Long term goal)
- ### Rendering API support
	- [x] OpenGL
	- [x] Vulkan
	- [x] Metal
	- [x] DirectX11
	- [x] DirectX12
- ### Crates
  - [x] (WIP) lambda-args -- Command line argument parsing.
  - [x] (WIP) lambda-platform -- Dependency wrappers & platform support.
  - [ ] lambda-cloud -- Cloud integrations.
  - [x] (WIP) lambda -- The public Lambda API.
- ### Examples
  - [x] Minimal -- A minimal example of an application with a working window
  using lambda.
  - [x] Push Constants -- An example of using shaders with push constants to
  render a 3D image.
  - [x] Triangle -- An example using shaders to render a single triangle.
  - [x] Triangles -- An example using shaders to render multiple triangles and keyboard input to move one of the triangles on screen.
- ### Tools
  - [x] obj-loader -- (WIP) Loads .obj files into lambda. Meshes need to be triangulated in order for it to render at the moment.
  - [ ] platform-info -- Utility for viewing information about the current platform.
- ### CI/CD
  - [x] Github action pipelines for building lambda on all platforms.
  - [ ] Github action pipelines for creating downloadable builds from releases.
  - [ ] Tests & benchmarking.
    - [ ] Unit tests.
  - [ ] Nightly builds.

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
