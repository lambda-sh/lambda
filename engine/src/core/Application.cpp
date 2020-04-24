#include "core/Application.h"

#include <functional>
#include <memory>

#include "core/Input.h"
#include "core/Layer.h"
#include "core/Log.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"

namespace engine {

Application* Application::kApplication_ = nullptr;

// Will only allow one application to be created per engine process.
Application::Application() {
  ENGINE_CORE_ASSERT(!kApplication_, "Application already exists.");
  kApplication_ = this;

  window_ = std::unique_ptr<Window>(Window::Create());
  window_->SetEventCallback(BIND_EVENT_FN(Application::OnEvent));
}

Application::~Application() {}

// Specifically retrieve updates from the window first to dispatch input
// events to every layer before making updates to them.
// TODO(C3NZ): Check to see which kind of updates need to come first and what
// the performance impact of each are.
void Application::Run() {
  while (running_) {
    std::pair<float, float> position = Input::GetMousePosition();
    ENGINE_CORE_TRACE("{0}, {1}", position.first, position.second);

    window_->OnUpdate();
    for (Layer* layer : layer_stack_) {
      layer->OnUpdate();
    }
  }
}

void Application::PushLayer(Layer* layer) {
  layer_stack_.PushLayer(layer);
  layer->OnAttach();
}

void Application::PushOverlay(Layer* layer) {
  layer_stack_.PushOverlay(layer);
  layer->OnAttach();
}

bool Application::OnWindowClosed(const events::WindowCloseEvent& event) {
  running_ = false;
  return true;
}

// This is the primary handler for passing events generated from the window back
// into the our application and game. Eventuaa
void Application::OnEvent(events::Event* event) {
  events::EventDispatcher dispatcher(event);
  dispatcher.Dispatch<events::WindowCloseEvent>
      (BIND_EVENT_FN(Application::OnWindowClosed));
  ENGINE_CORE_TRACE(*event);

  // Pass the event to all needed layers on the stack.
  for (auto it = layer_stack_.end(); it != layer_stack_.begin();) {
    (*--it)->OnEvent(event);
    if (event->HasBeenHandled()) {
      break;
    }
  }
}

}  // namespace engine
