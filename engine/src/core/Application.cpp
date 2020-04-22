#include "core/Application.h"

#include <functional>

#include "core/Log.h"
#include "core/Window.h"
#include "core/Layer.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"

namespace engine {

// Bind an application event handler to an application instance to be passed
// into functions that use callbacks for returns.
#define BIND_EVENT_FN(handler) \
  std::bind(&Application::handler, this, std::placeholders::_1)

Application* Application::kApplication_ = nullptr;

Application::Application() {
  ENGINE_CORE_ASSERT(!kApplication_, "Application already exists.");
  kApplication_ = this;

  window_ = std::unique_ptr<Window>(Window::Create());
  window_->SetEventCallback(BIND_EVENT_FN(OnEvent));
}

Application::~Application() {}


void Application::Run() {
  while (running_) {
    // Update every layer
    for (Layer* layer : layer_stack_) {
      layer->OnUpdate();
    }

    window_->OnUpdate();
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

// Event dispatcher for handling events.
void Application::OnEvent(events::Event* event) {
  events::EventDispatcher dispatcher(event);
  dispatcher.Dispatch<events::WindowCloseEvent>(BIND_EVENT_FN(OnWindowClosed));
  ENGINE_CORE_TRACE(*event);

  for (auto it = layer_stack_.end(); it != layer_stack_.begin();) {
    (*--it)->OnEvent(event);
    if (event->HasBeenHandled()) {
      break;
    }
  }
}

}  // namespace engine
