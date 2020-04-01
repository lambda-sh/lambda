#include "Application.h"

#include <iostream>

namespace engine {
  Application::Application() {

  }

  Application::~Application() {

  }

  void Application::Run() {
    while (true) {
      std::cout << "Hello";
    }
  }
}
