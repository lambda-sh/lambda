#include "Engine.h"

class ExampleLayer : public engine::Layer {
 public:
  ExampleLayer() : Layer("Example") {}

  void OnUpdate() override {
    ENGINE_CLIENT_INFO("ExampleLayer::Update");
  }

  void OnEvent(engine::events::Event* event) override {
    ENGINE_CLIENT_TRACE("{0}", *event);
  }
};

class Sandbox : public engine::Application {
 public:
  Sandbox() {
    PushLayer(new ExampleLayer());
  }

  ~Sandbox() {}
};

engine::Application* engine::CreateApplication() { return new Sandbox(); }
