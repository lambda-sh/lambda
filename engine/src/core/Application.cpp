#include "Application.h"

#include <functional>

#include "Log.h"
#include "Window.h"
#include "events/Event.h"
#include "events/ApplicationEvent.h"

namespace engine {

// Bind an application event handler to an application instance to be passed
// into functions that use callbacks for returns.
#define BIND_EVENT_FN(handler) \
  std::bind(&Application::handler, this, std::placeholders::_1)

Application::Application() {
  window_ = std::unique_ptr<Window>(Window::Create());
  window_->SetEventCallback(BIND_EVENT_FN(OnEvent));
}

Application::~Application() {}

void Application::Run() {
  while (running_) {
    window_->OnUpdate();
  }
}


bool Application::OnWindowClosed(const events::WindowCloseEvent& event) {
  running_ = false;
  return true;
}

// Event dispatcher for handling events.
void Application::OnEvent(const events::Event& event) {
  events::EventDispatcher dispatcher(event);
  dispatcher.Dispatch<events::WindowCloseEvent>(BIND_EVENT_FN(OnWindowClosed));
  ENGINE_CORE_INFO(event);
}

}  // namespace engine
