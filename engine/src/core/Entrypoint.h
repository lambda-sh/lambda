/**
 * @file Entrypoint.h
 * @brief The entrypoint into the engine.
 *
 * It defines the CreateApplication function as an external function that is to
 * be implemented in the application.
 */
#ifndef ENGINE_SRC_CORE_ENTRYPOINT_H_
#define ENGINE_SRC_CORE_ENTRYPOINT_H_

#include "core/Application.h"
#include "core/util/Log.h"

#ifdef ENGINE_PLATFORM_LINUX

extern lambda::core::Application* lambda::core::CreateApplication();

int main() {
  lambda::core::util::Log::Init();
  ENGINE_CORE_WARN("Initialized core log");
  ENGINE_CLIENT_INFO("Initialized client log");

  auto app = lambda::core::CreateApplication();
  app->Run();
  delete app;

  ENGINE_CLIENT_INFO("Game has been closed");
  return 0;
}

#endif  // ENGINE_PLATFORM_LINUX
#endif  // ENGINE_SRC_CORE_ENTRYPOINT_H_
