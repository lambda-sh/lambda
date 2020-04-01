#ifndef SRC_CORE_ENTRYPOINT
#define SRC_CORE_ENTRYPOINT

#ifdef ENGINE_PLATFORM_LINUX

#include <iostream>

extern engine::Application* engine::CreateApplication();

int main() {
  auto app = engine::CreateApplication();
  app->Run();
  delete app;
  return 0;
}

#endif  // ENGINE_PLATFORM_LINUX

#endif  // SRC_CORE_ENTRYPOINT
