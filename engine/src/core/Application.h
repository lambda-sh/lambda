#ifndef ENGINE_SRC_CORE_APPLICATION_H_
#define ENGINE_SRC_CORE_APPLICATION_H_

#include "Core.h"
#include "Window.h"
#include "events/ApplicationEvent.h"
#include "events/Event.h"

namespace engine {

class ENGINE_API Application {
 public:
  Application();
  virtual ~Application();

  void Run();
  void OnEvent(events::Event* event);
 private:
  bool running_ = true;
  std::unique_ptr<Window> window_;

  bool OnWindowClosed(const events::WindowCloseEvent& event);
};

// To be defined in client.
Application* CreateApplication();

}  // namespace engine

#endif  // ENGINE_SRC_CORE_APPLICATION_H_
