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
    2. [With Cmake](#cmake)
    3. [In Visual studio](#vs) 
4. [Directory structure](#dirs)
5. [Getting started](#get_started)
    1. [Basic application](#basic_app)
6. [Planned additions](#plans)
7. [How to contribute](#contribute)
8. [Resources](#resources)
## Description <a name="description"></a>
More features will come along as the project develops.
Lambda is a framework for developing cross platform applications using modern 
C++.

Lambda aims to enable developers to create highly performant, portable, and 
minimal desktop applications by providing a platform agnostic API for just about 
anything an application might need.

Lambda aims to enable an easy and efficient way to create:
* Desktop applications
* Productivity tools
* Data visualizations
* Physical simulations
* Games

While lambda is still in beta, we're making sure to create a solid 
foundation for the project to build upon. Over the last couple of years, the 
prominence of the web has replaced traditional desktop applications. While 
this has lead to an overall increase in UI/UX for applications across all 
platforms, it has come at the cost of performance & resource consumption. 

Lambda's goal isn't to replace electron and similar web based desktop 
applications, however; it is to instead create a cross platform ecosystem for 
desktop applications with performance and resource consumption at the forefront 
of it's priorities. Lambda will not be HTML/CSS based and will instead provide 
it's own tooling 

## API Documentation <a name="documentation"></a>

All documentation can be found at [Lambda-docs](https://engine-docs.cenz.io).
Documentation is being written as the engine progresses, so it may currently
lack behind what the engine is capable of.

All documentation is generated using doxygen. Feel free to contribute to areas
of the codebase that currently don't have documentation or dont use the majority
style.

## Building <a name="building"></a>

### From source <a name="source"></a>
Currently, building from source is the only method to obtain a copy of Lambda.
However, all of the API depdendencies are provided by Lambda and there are a few
external dependencies (Like opengl, vulkan, cmake) that are needed to build this
project.

#### External dependencies <a name="ext_deps"></a>
* `cmake` is needed to generate project files. Version `3.20.0` is required.
* `opengl` is needed for rendering.
  * If working on ubuntu, `xrandr-dev` is needed.
* `git` is needed to clone the project and manage it's depdencies.
* `git-lfs` is needed for asset files.
* `gcc`, `clang`, or `msvc` are the supported c++ compilers and Lambda is built
using `c++20` as it's standard.

#### Optional dependencies <a name="opt_deps"></a>
* `bash` is not necessary needed to build the project, but makes doing so a 
whole lot simpler as there are multiple scripts for things like setup, building,
and running tools packaged with Lambda.
* `ninja` is what bash scripts `compile_and_run.sh` & `compile_lambda.sh` use to
build Lambda and tools.
* `gdb` is great for if you need to debug builds of your application with 
Lambda and what I use for development.
* `vscode` I have plans to add environment build and run configurations for 
vscode to make setting up the development environment built upon the scripts
provided in `scripts`.

#### Linux (bash), MacOS (bash), Windows (git-bash) <a name="bash"></a>
All dependencies for Lambda (outside of the ons listed in pre-reqs) are
packaged with Lambda inside of `lambda/vendor` as git submodules. In order to 
fetch these submodules and obtain their contents, you can run:
```bash
./scripts/setup.sh
```
This will initialize pre commit checks for development use and also 
initialize all git submodules. If you're not doing development, you can just use
the commands within `setup.sh` that clone the submodules.

In order to build and run a release version of the sandbox that comes with the
engine, you can simply run:
```bash
./scripts/compile_and_run.sh
```
This will run the tool `sandbox` located within `tools/sandbox/sandbox`. This 
tool is just for testing out the features that are being worked on within the 
engine. This script requires `cmake`, `ninja`, and a valid c++ compiler

In order to change the build type, you can utilize:
```bash
./scripts/compile_and_run.sh --build Debug
```

To learn more about what scripts offer, try passing --help into them like so:
```bash
./scripts/compile_and_run.sh --help
```

### with cmake. <a name="cmake"></a>

Currently, Lambda is built using cmake. In order to add this to your own
project that also was using cmake, you could use these commands:
```
# In your CMakeLists.txt file...

add_subdirectory(dependency_dir/lambda)
target_link_libraries(your_app lambda)
target_include_directories(sandbox PRIVATE "${LAMBDA_ENGINE_INCLUDE_DIR}")
```

In the above example, `dependency_dir` is where your dependencies are being 
pulled from and Lambda is this repo either as a clone or submodule of the repo 
you're working out of. This should be relative and below the cmake file (Like
how lambda structures it's dependencies.) `your_app` should be the name of the
executable that you're trying to build.

Once that's done, you should now be able to include <Lambda.h> into your code
and now should have access to the `lambda` namespace.

### in visual studio <a name="vs"></a>
Assuming that you've fetched all of the project dependencies and have Lambda 
checked out in your file system, you should just be able to open the project
with Visual studio to start running different builds.

Visual studio needs to have cmake to generate the project but project generation
pre configured in `CMakeSettings.json` for giving you access to install or 
run varying builds of lambda.

## Directory structure <a name="dirs"></a>

`lambda` - The source code, dependencies, and assets for the lambda engine.
* `assets` - Assets packaged with lambda. (Stored with git-lfs)
* `src`
  * `Lambda`
    * `concepts` - C++20 concept headers for template parameter constrains 
    defined for templates within lambda
    * `core` - The core api.
      * `events` - Event handling library with a predefined set of common 
      events.
      * `imgui` - imgui rendering layer for implementing dynamic user 
      interfaces. (Currently only supports opengl rendering)
      * `input` - Input listening and API keycodes.
      * `io` - (Experimental) an lockless event loop that can sit in a thread 
      and dispatch functions. 
      * `layers` - a layering system for encapsulating UI logic and event 
      handling within a container that sits within an `Application`
      * `memory` - (Experimental) the memory system for allocating memory within
      the engine.
      * `renderer` - The rendering API for drawing in lambda.
    * `lib` - Useful code that is used across all components of lambda.
    * `math` - The math API.
      * `plot` - Items that are plottable by lambda.
      * `shapes` - API to represent geometric shapes
    * `platform` - Dependency wrappers that are safe to use within lambda for 
    (almost) all of the dependencies that are provided.
      * `glad` - GL extension loader. Supports up to OpenGL 4.5.
      * `glfw` - Wrapper over GLFW with define included.
      * `linux` - Linux specific API implementations.
      * `macos` - MacOS specific API implementations.
      * `opengl` - OpenGL specific rendering implementations.
      * `windows` - Windows specific API implementations.
    * `profiler` - a simple intrusive profiler that records runtime execution 
    times where requested and exports data as JSON that is able to be rendered
    within chrome tracing tools.
  * `vendor` - Dependencies shipped with lambda. (Stored as git submodules)
    * `concurrentqueue` - Is an atomic queue implementation used for the event 
    loop. It supports one publisher with many subscribers.
    * `glad` - OpenGL 4.5 extension loader for all platforms.
    * `glfw` - Windowing system for lambda.
    * `glm` - OpenGL mathematics API for rendering. Will be replaced by 
    `Lambda/math`.
    * `googletest` - Google testing framework for tests I haven't wrote yet. :)
    * `imgui` - UI components that are cross platform.
    * `readerwriterqueue` - An atomic queue that supports one publisher and one
    subscriber.
    * `stb` - Used for loading images.

`docs` - the documentation server and static content for lambda.
Primarily used internally.

`tools` - C++ tools that are made from the engine for testing it and
experimenting with it's potential.
* `mathbox` - Tool creating, using, & profiling the performance of the math API.
  * `assets` - Assets for mathbox
  * `src` - Source of mathbox
* `sandbox` - Tool for testing out the rendering API.
  * `assets` - Assets for sandbox
  * `src` - Source of sandbox

`scripts` - bash scripts used for automating tedious tasks. (Like compiling and
running tools)

## Getting started <a name="get_started"></a>

There are two components that are required to know for getting started with 
Lambda.

1. **Application** - This class serves as the base for your application and 
are the engine that powers all applications built on top on lambda. It is 
located at `lambda::core::Application`.

2. **Layers** - Layers serve as containers for the logic of your application 
that can be hooked into the Application for running your code. They provide the
hooks for updating state, rendering, event handling, and Init/Destructin logic
to be run by the engine. It is located at `lambda::core::layers::Layer`.


### Basic Application <a name="basic_app"></a>

```c++
// in some file maybe named example.cpp
#include <Lambda/Lambda.h>
#include <Lambda/core/Entrypoint.h>

using namespace lambda::core;
using namespace lambda::lib;

// Our Layer to receive events and hook into the update loop within lambda. You
// can make as many layers as you like!
class HelloLayer : layers::Layer {
  public:
    // OnUpdate provides you when the last update occurred as a delta that 
    // can be computed as whatever precision is needed.
    void OnUpdate(lib::TimeStep delta) override {
      LAMBDA_CLIENT_LOG(
          "{} seconds since last update.", delta.InSeconds<double>());
    }

    // Provided by the Application, Events are generic pointers that are 
    // used for handling more specific types of events using the 
    // events::Dispacther.
    void OnEvent(Event* const event) override {}
};

// Our Application instance.
class HelloLambda : public Application {
  public:
    // The constructor serves as your application's way to initialize the state 
    // of your application before running.
    HelloLambda() : Application() {
      PushLayer(memory::CreateUnique<HelloLayer>());
    }
};

// This function becomes your new main function. Any logic here is used before 
// Creating an instance of your Application (In this case, HelloLambda). Lambda
// needs this function implemented and returning a valid Application instance
// order to instantiate 
memory::Unique<Application> lambda::core::CreateApplication() {
  return memory::CreateUnique<HelloLambda>(); 
}
```
Assuming you've followed the build steps above or are linking against lambda 
manually, you should now have an instance of a lambda powered application 
running on your computer.

We're currently working on more working examples and will be focusing on 
documentation as the library becomes more stable and developed. There will also
be several free tools provided with the engine that is what is currently used
for testing features and benchmarking.

## Planned additions <a name="plans"></a>

- [ ] c++20 concepts for all template parameters. This will limit template 
parameter usage within lambda to an explicitly defined set of types that can 
be built on top of eachother. For example, `lambda::concepts` currently provides
the concept:
  ```c++
  template<class MaybeContainer>
  concept NumberContainer = NumberArray<MaybeContainer>
    || NumberVector<MaybeContainer>
  ```
  The concept `NumberContainer` defines whether the type 
  `MaybeContainer` fits the `concept` of being either a `std::array` or 
  `std::vector` of numbers. Implementation can be found within the tool 
  `mathbox` and is also currently used throughout `lambda/src/Lambda/math`.
- [ ] A better memory model that's consistent across all APIs and STL 
compatible. Currently the `memory` module just wraps smart pointers and there 
are parts of the API that create some extraneous references to shared pointers. 
While it currently doesn't impact lambda too much, the ideal goal is to 
have an allocator built into Lambda.
- [ ] Github action pipelines for creating downloadable builds from releases.
- [ ] An extensively featured declarative math library.
- [ ] Extensive support for 2D graphics. 3D will most likely come a bit after.
- [ ] A functional UI component system for building extensible & consistent 
user interfaces that are native towards every system. Currently, 
lambda provides `imgui` support with opengl which provides a lot of 
functionality, but requires also learning about how imguis API work.
- [ ] Test & benchmarking setup. Might make a profiling layer that can be 
included into applications for debugging the internals of the Application 
while it's running.

Formal feature additions or fixes will be tracked as issues.

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

[Creator of the logo's github.](https://github.com/RinniSwift)
