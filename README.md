<p align="center" style="text-align:center;">
  <img src="https://github.com/C3NZ/lambda/blob/master/lambda/assets/logos/lambda_header.png">
</p>

## Description
This repo is for learning how to structure and make a game engine! This
repository is inspired by The Cherno's game engine series and is especially
reliant on the rendering engine that is implemented in Hazel. If you'd like
to learn more about the awesome series, please check out the link to his
channel & series down below.

lambda on the other hand is my own take at a cross platform game engine that is
is meant to be entirely free and easy to use. It is also for me to further my
knowledge in C++.

## How to run/setup
In order to get external dependencies that are needed in order to run the project,
please run:
```
./scripts/setup.sh
```
This will initialize pre commit checks for development and also initialize all
git submodules.

In order to build and run a release version of the sandbox that comes with the
engine, you can simply run:
```bash
./scripts/compile_and_run.sh
```

In order to change the build, you can utilize:
```bash
./scripts/compile_and_run.sh --build Debug
```

To learn more about what scripts offer, try passing --help into them like so:
```bash
./scripts/compile_and_run.sh --help
```

## Documentation
All documentation can be found at [lambda-docs](https://engine-docs.cenz.io).
Documentation is being written as the engine progresses, so it may currently
lack behind what the engine is capable of.

All documentation is generated using doxygen. Feel free to contribute to areas
of the codebase that currently don't have documentation or dont use the majority
style.

## Directory structure

`lambda` -- The source code, dependencies, and assets for the lambda engine.

`docs` -- the documentation server and static content for lambda.
Primarily used internally.

`tools` -- C++ tools that are made from the engine for testing it and
experimenting with it's potential.

`scripts` -- bash scripts used for automating tedious tasks. (Like compiling and
running tools)

## Getting started

### Support

Currently only linux is being supported and hasn't been tested on other
platforms. A very useful type of contribution to this project would be to verify
and potentially extend the current API to support as many platforms and graphics
APIs as possible, but getting lambda to work, be highly efficient, and be well
thought out is the current plan to avoid as much technical debt as possible in
the future.

### Building

Currently, lambda is built using cmake. In order to add this to your own
project that also was using cmake, you could use these commands:
```
# In your CMakeLists.txt file...

add_subdirectory(dependency_dir/lambda)
target_link_libraries(your_app lambda)
```

In the above example, `dependency_dir` is where your dependencies and most likely
lambda will be setting. This should be relative and below the cmake file (Like
how lambda structures it's dependencies.) `your_app` should be the name of the
executable that you're trying to build.

Once that's done, you should now be able to include <Lambda.h> into your code
and now should have access to the `lambda` namespace.

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
#include <Lambda.h>

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

using lambda::core::util::TimeStep;

// Our Layer to receive events and hook into the update loop within lambda. You
// can make as many layers as you like!
class HelloLayer : Layer {
  public:
    // Hook into Lambdas update loop!
    void OnUpdate(TimeStep delta) override {
      LAMBDA_CLIENT_LOG(
          "{} seconds since last update." delta<double>.InSeconds());
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

And boom, this starter pack will get you on your feet and running! While this
tutorial doesn't cover the majority of the library, there will be more tutorials
like this. Please refer to the documentation if need to figure out how to use
the API. It is regularly updated.

## How to contribute
Fork the current repository and then make the changes that you'd like to said fork. Upon adding features, fixing bugs,
or whatever modifications you've made to the project, issue a pull request to this repository containing the changes that you've made
and I will evaluate them before taking further action. This process may take anywhere from 3-7 days depending on the scope of the changes made,
my schedule, and any other variable factors.

## Resources
[The Cherno's playlist for making a game engine](https://www.youtube.com/playlist?list=PLlrATfBNZ98dC-V-N3m0Go4deliWHPFwT)

[Creator of the logo's github. (Huge shoutout)](https://github.com/RinniSwift)
