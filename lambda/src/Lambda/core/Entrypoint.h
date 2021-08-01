/// @file Entrypoint.h
/// @brief The entrypoint into the engine.
///
/// It defines the CreateApplication function as an external function that is to
/// be implemented in the application.

#ifndef LAMBDA_SRC_LAMBDA_CORE_ENTRYPOINT_H_
#define LAMBDA_SRC_LAMBDA_CORE_ENTRYPOINT_H_

#include <Lambda/core/Application.h>
#include <Lambda/lib/Log.h>

using lambda::core::Application;
using lambda::core::memory::Unique;

/// This is to be defined externally in the application that.
extern Unique<Application> lambda::core::CreateApplication();

int main() {
  lambda::lib::Log::Init();
  LAMBDA_CORE_WARN("Initialized core log");
  LAMBDA_CLIENT_INFO("Initialized client log");

  auto app = lambda::core::CreateApplication();
  app->Run();

  LAMBDA_CLIENT_INFO("Game has been closed");
  return 0;
}

#endif  // LAMBDA_SRC_LAMBDA_CORE_ENTRYPOINT_H_
