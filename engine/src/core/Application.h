#ifndef ENGINE_CORE_APPLICATION_H_
#define ENGINE_CORE_APPLICATION_H_

#include "Core.h"

namespace engine {

  class ENGINE_API Application {
    public:
      Application();
      virtual ~Application();

      void Run();
  };

  // To be defined in client.
  Application* CreateApplication();
}
#endif
