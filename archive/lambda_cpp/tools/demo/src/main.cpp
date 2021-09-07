#include "Lambda/Lambda.h"
#include "Lambda/core/Entrypoint.h"

using namespace lambda::core;
using namespace lambda::lib;

// Our Layer to receive events and hook into the update loop within lambda. You
// can make as many layers as you like!
class HelloLayer : public layers::Layer {
  public:
    HelloLayer(){};
    // OnUpdate provides you when the last update occurred as a delta that
    // can be computed as whatever precision is needed.
    void OnUpdate(TimeStep delta) override {
      LAMBDA_CLIENT_INFO("{} seconds since last update.",
                         delta.InSeconds<double>());
    }

    // Provided by the Application, Events are generic pointers that are
    // used for handling more specific types of events using the
    // events::Dispacther
    void OnEvent(events::Event *const event) override{};

    void OnAttach() override{};

    void OnDetach() override{};

    void OnImGuiRender() override{};
};

// Our Application instance.
class HelloLambda : public Application {
  public:
    // The constructor servers as your application's way of initializing the state
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
