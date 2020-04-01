#ifndef ENGINE_CORE_ENTRYPOINT_H_
#define ENGINE_CORE_ENTRYPOINT_H_

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
