<p align="center" style="text-align:center;">
<img src="https://media.githubusercontent.com/media/lambda-sh/lambda/main/lambda/assets/logos/lambda_header.png" />
</p>

## Table of contents
1. [Description](#description)
2. [API Documentation](#documentation)
3. [Building](#building)
    1. [From source](#source)
        1. [External dependencies](#ext_deps)
        2. [Optional depedencies](#opt_deps)
        3. [Linux, Macos, Windows](#bash)
5. [Getting started](#get_started)
6. [Planned additions](#plans)
7. [How to contribute](#contribute)
8. [Resources](#resources)
## Description <a name="description"></a>
Lambda is a framework for developing cross platform applications and workloads using Rust.

Lambda aims to enable developers to create highly performant, portable, and
minimal desktop applications by providing a platform agnostic API for all of the features that any application or workload might need.

Lambda :
* Desktop applications
* Productivity tools
* Data visualizations
* Physical simulations
* Games

While lambda is still in beta, we're making sure to create a solid
foundation other projects to build upon. Over the last couple of years, the
prominence of the web has replaced traditional desktop applications. While
this has lead to a golden age for developing UI/UX for applications across all
platforms, it has come at the cost of degraded performance & resource consumption.

Lambda's goal isn't to replace electron, webview, or other similar web based desktop
frameworks, however; it is to instead create a cross platform ecosystem for
desktop applications with performance and resource consumption at the forefront
of it's priorities without sacrificing. Lambda will not be HTML/CSS based and will instead provide

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
These are the Rendering APIs that are supported on each platform and must be installed manually. More information on how to choose which backend lambda uses on each platform is provided further below.
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


## Plans <a name="plans"></a>
- [ ] Architecture support
	- [x] x86
	- [ ] arm64
- [ ] Operating system support
	- [x] MacOS
	- [x] Linux
	- [x] Windows 10/11
	- [ ] Xbox Series S|X (Long term goal)
	- [ ] iOS (Long term goal)
	- [ ] Android (Long term goal)
- [x] Rendering API support
	- [x] OpenGL
	- [x] Vulkan
	- [x] Metal
	- [x] DirectX11
	- [x] DirectX12
- [ ] Crates
	- [ ] lambda-arch -- Architecture support
	- [ ] (WIP) lambda-platform -- Platform support
	- [ ] (WIP) lambda-core -- Core library implementations
	- [ ] lambda-cloud -- Cloud integrations
	- [ ] (WIP) lambda --  Stable Lambda API
- [ ] Tools
	- [ ] lambda-rs-demo -- 2D rendering demo
	- [ ] lambda-rs-cube -- 3D rendering demo
	- [ ] lambda-checker -- Checks system specifications against lambda requirements
- [ ] CI/CD
	- [ ] Github action pipelines for creating downloadable builds from releases.
	- [ ] Tests & benchmarking.

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

## Resources I used for creating this project. <a name="resources"></a>
[The Cherno's playlist for making a game engine](https://www.youtube.com/playlist?list=PLlrATfBNZ98dC-V-N3m0Go4deliWHPFwT)

[Creator of the repository logo.](https://github.com/RinniSwift)
