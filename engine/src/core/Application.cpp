#include "Application.h"

#include "core/Log.h"
#include "core/Window.h"

namespace engine {

Application::Application() {
  window_ = std::unique_ptr<Window>(Window::Create());
}

Application::~Application() {}

void Application::Run() {
  while (running_) {
    window_->OnUpdate();
  }
}

}  // namespace engine
