#ifndef SRC_CORE_APPLICATION
#define SRC_CORE_APPLICATION

namespace engine {
  class Application {
    public:
      Application();
      virtual ~Application();

      void Run();
  };

  // To be defined in client.
  Application* CreateApplication();
}
#endif
