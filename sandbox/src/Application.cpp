#include "Engine.h"

class ExampleLayer : public engine::Layer {
 public:
  ExampleLayer() : Layer("Example") {}

  void OnUpdate() override {
    // ENGINE_CLIENT_INFO("ExampleLayer::Update");
    if (engine::Input::IsKeyPressed(ENGINE_KEY_TAB)) {
        ENGINE_CLIENT_INFO("Tab key is pressed!");
      }
  }

  void OnEvent(engine::events::Event* event) override {
    // ENGINE_CLIENT_TRACE("{0}", *event);
  }
};

class Sandbox : public engine::Application {
 public:
  Sandbox() {
    PushLayer(new ExampleLayer());
    PushLayer(new engine::imgui::ImGuiLayer());
  }

  ~Sandbox() {}
};

engine::Application* engine::CreateApplication() { return new Sandbox(); }
