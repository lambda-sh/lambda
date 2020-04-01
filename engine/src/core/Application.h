#ifndef SRC_CORE_APPLICATION
#define SRC_CORE_APPLICATION

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
