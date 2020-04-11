#include "Application.h"

#include "core/Log.h"
#include "core/events/ApplicationEvent.h"

namespace engine {
  Application::Application() {}

  Application::~Application() {}

  void Application::Run() {
    events::WindowResizeEvent e(1280, 720);
    ENGINE_CORE_TRACE(e);
    while (true) {}
  }
}  // namespace engine
