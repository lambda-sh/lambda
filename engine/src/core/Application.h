#ifndef ENGINE_SRC_CORE_APPLICATION_H_
#define ENGINE_SRC_CORE_APPLICATION_H_

#include "Core.h"
#include "Window.h"

namespace engine {

class ENGINE_API Application {
 public:
  Application();
  virtual ~Application();

  void Run();
 private:
  bool running_ = true;
  std::unique_ptr<Window> window_;
};

// To be defined in client.
Application* CreateApplication();

}  // namespace engine

#endif  // ENGINE_SRC_CORE_APPLICATION_H_
