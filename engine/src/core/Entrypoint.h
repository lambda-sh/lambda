#ifndef SRC_CORE_ENTRYPOINT
#define SRC_CORE_ENTRYPOINT

extern engine::Application* engine::CreateApplication();

int main(int argc, char** argv) {
  auto app = engine::CreateApplication();
  app->Run();
  delete app;
}

#endif
