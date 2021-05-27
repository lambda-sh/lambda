<p align="center" style="text-align:center;">
<img src="https://media.githubusercontent.com/media/lambda-sh/lambda/main/lambda/assets/logos/lambda_header.png" />
</p>

## Description
lambda is a framework for developing cross platform applications using modern 
C++.

Lambda aims to enable developers to create highly performant, portable, and 
minimal desktop applications by providing a platform agnostic API for just about 
anything an application might need.

lambda will enable people to create:
* Desktop applications
* Productivity tools
* Data visualizations
* Physical simulations
* Games

While lambda is still in beta, we're making sure to create a solid 
foundation for the project to build upon. Over the last couple of years, the 
prominence of the web has replaced desktop applications. While this has lead to
an overall increase in UI/UX for applications across all platforms, it has come
at the cost of performance & resource consumption. 

lambda's goal isn't to replace electron and similar web based desktop 
applications however, it is to instead create a cross platform ecosystem for 
desktop applications with performance and resource consumption at the forefront 
of it's priorities.

## API Documentation
All documentation can be found at [lambda-docs](https://engine-docs.cenz.io).
Documentation is being written as the engine progresses, so it may currently
lack behind what the engine is capable of.

All documentation is generated using doxygen. Feel free to contribute to areas
of the codebase that currently don't have documentation or dont use the majority
style.

## Building from source
Currently, building from source is the only method to obtain a copy of lambda.
However, all of the API depdendencies are provided by lambda and there are a few
external dependencies (Like opengl, vulkan, cmake) that are needed to build this
project.

### Pre requisites regardless of OS
* `cmake` is needed to generate project files. Version `3.20.0` is required.
* `opengl` is needed for rendering.
  * If working on ubuntu, `xrandr-dev` is needed.
* `git` is needed to clone the project and manage it's depdencies.
* `git-lfs` is needed for asset files.
* `gcc`, `clang`, or `msvc` are the supported c++ compilers and lambda is built
using `c++20` as it's standard.

### Optional
* `bash` is not necessary needed to build the project, but makes doing so a 
whole lot simpler as there are multiple scripts for things like setup, building,
and running tools packaged with lambda.
* `ninja` is what bash scripts `compile_and_run.sh` & `compile_lambda.sh` use to
build lambda and tools.
* `gdb` is great for if you need to debug builds of your application with 
lambda and what I use for development.
* `vscode` I have plans to add environment build and run configurations for 
vscode to make setting up the development environment built upon the scripts
provided in `scripts`.

### Linux, MacOS, Windows (git-bash)
All dependencies for lambda (outside of the ons listed in pre-reqs) are
packaged with lambda inside of `lambda/vendor` as git submodules. In order to 
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
## Building with a Cmake project

Currently, lambda is built using cmake. In order to add this to your own
project that also was using cmake, you could use these commands:
```
# In your CMakeLists.txt file...

add_subdirectory(dependency_dir/lambda)
target_link_libraries(your_app lambda)
```

In the above example, `dependency_dir` is where your dependencies are being 
pulled from and lambda is this repo either as a clone or submodule of the repo 
you're working out of. This should be relative and below the cmake file (Like
how lambda structures it's dependencies.) `your_app` should be the name of the
executable that you're trying to build.

Once that's done, you should now be able to include <Lambda.h> into your code
and now should have access to the `lambda` namespace.

## Building in Visual studio
Assuming that you've fetched all of the project dependencies and have lambda 
checked out in your file system, you should just be able to open the project
with Visual studio to start running different builds.

Visual studio needs to have cmake to generate the project but project generation
pre configured in `CMakeSettings.json` for giving you access to install or 
run varying builds of lambda.

## Directory structure

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

### Your first application
There are two components that are required to know with hooking into Lambda.

1. Application
2. Layers

The `Application` class within `lambda::core::Application` is what allows you to
hook lambda in order for it to power all of your applications. By creating a a
class that extends from `Application`, you're provided with a set of
functionality that allows you to automatically add `Layers` of code to the
engine that do exactly what you need to do via `PushLayer(new Layer())`. Let's
look at a simple program that hooks into lambda and does exactly that:

```c++
// This gets you all of lambdas core. (Minus the entrypoint)
#include <Lambda/Lambda.h>
// This is required by the entrypoint of your application file. This is where
// lambda::core::CreateApplication gets run to create your instance!
#include <Lambda/core/Entrypoint.h>

// Gives us access to CreateShared<Class T>(...);
// The engine rarely does work with raw pointers and unless you have good reason
// for doing so, neither should you!
using lambda::core::memory::CreateShared;

// Gives us access to Unique<Class T>, which is just a smart pointer at the
// moment (Will be changed in the future but should be used right now.)
using lambda::core::memory::Unique

// Gives us access to Shared<Class T>, which is just a smart pointer at the
// moment (Will be changed in the future but should be used right now.)
using lambda::core::memory::Shared;

// Gives us access to the Application API
using lambda::core::Application;

// Gives us access to the generic layer API
using lambda::core::layers::Layer;

using lambda::lib::TimeStep;

// Our Layer to receive events and hook into the update loop within lambda. You
// can make as many layers as you like!
class HelloLayer : Layer {
  public:
    // Hook into Lambdas update loop!
    void OnUpdate(TimeStep delta) override {
      LAMBDA_CLIENT_LOG(
          "{} seconds since last update.", delta.InSeconds<double>());
    }

    // Hook into Lambda's event system!
    void OnEvent(Shared<Event> event) override {

    }
};

// Our Application instance. We can only ever instantiate one of these at a
// time! (Will fail if LAMBDA_ASSERT_ENABLES is true.)
class HelloLambda : public Application {
  public:
    // Push the HelloLayer into the Application. The layer and your logic now
    // have access to events and the update loop!
    HelloLambda() {
      PushLayer(CreateShared<HelloLayer>());
    }
};

// Once you've created an application and a layer, you now have the ability to
// spin up your application with:
Unique<Application> lambda::core::CreateApplication() {
  return CreateUnique<HelloLambda>(); }
```

This starter pack gets you into the lambda engine which now allows to hook into
update & event system that lambda provides for your application and layers. 

We're currently working on more working examples and will be focusing on 
documentation as the library becomes more stable and developed.

## How to contribute
Fork the current repository and then make the changes that you'd like to 
said fork. Stable releases will happen within the main branch requiring that 
additions to be made off of `dev` which is the nightly build branch for lambda.
Upon adding features, fixing bugs, or whatever modifications you've made to the 
project, issue a pull request into the latest `dev` branch containing the 
changes that you've made and I will evaluate them before taking further action. 
This process may take anywhere from 3-7 days depending on the scope of the 
changes made, my schedule, and any other variable factors. They must also pass 
all of the build pipelines are configured to run at merge.

## Resources I used for creating this project.
[The Cherno's playlist for making a game engine](https://www.youtube.com/playlist?list=PLlrATfBNZ98dC-V-N3m0Go4deliWHPFwT)

[Creator of the logo's github.](https://github.com/RinniSwift)
