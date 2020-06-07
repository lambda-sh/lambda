#include "core/Application.h"

#include <functional>
#include <initializer_list>
#include <memory>

#include "core/Assert.h"
#include "core/Input.h"
#include "core/Layer.h"
#include "core/Log.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/renderer/Buffer.h"
#include "core/renderer/Shader.h"
#include "core/renderer/VertexArray.h"
#include "core/renderer/Renderer.h"
#include "core/renderer/RenderCommand.h"

namespace engine {

Application* Application::kApplication_ = nullptr;

/**
 * TODO(C3NZ): This should not carry as much of a load as it currently does
 * and should instead be delegated to applications attempting to use the engine.
 */
Application::Application() {
  ENGINE_CORE_ASSERT(!kApplication_, "Application already exists.");
  kApplication_ = this;

  window_ = std::unique_ptr<Window>(Window::Create());
  window_->SetEventCallback(BIND_EVENT_FN(Application::OnEvent));

  imgui_layer_ = new imgui::ImGuiLayer();
  PushLayer(imgui_layer_);

  // Setup our vertices.
  float vertices[3 * 7] = {
    -0.5f, -0.5f, 0.0f, 0.0f, 0.0f, 0.9f, 1.0f,
     0.5f, -0.5f, 0.0f, 0.0f, 0.0f, 1.0f, 1.0f,
     0.0f, 0.5f, 0.0f, 1.0f, 1.0f, 0.9f, 1.0f,
  };

  vertex_array_.reset(renderer::VertexArray::Create());

  vertex_buffer_.reset(
      renderer::VertexBuffer::Create(vertices, sizeof(vertices)));

  renderer::BufferLayout layout_init_list = {
      { renderer::ShaderDataType::Float3, "a_Position"},
      { renderer::ShaderDataType::Float4, "a_Color", true}};

  renderer::BufferLayout layout(layout_init_list);
  vertex_buffer_->SetLayout(layout);

  vertex_array_->AddVertexBuffer(vertex_buffer_);

  unsigned int indices[3] = { 0, 1, 2 };
  index_buffer_.reset(renderer::IndexBuffer::Create(indices, 3));

  vertex_array_->SetIndexBuffer(index_buffer_);

  std::string vertex_source = R"(
      #version 330 core

      layout(location = 0) in vec3 a_Position;
      layout(location = 1) in vec4 a_Color;

      out vec3 v_Position;
      out vec4 v_Color;

      void main() {
        v_Position = a_Position;
        v_Color = a_Color;
        gl_Position = vec4(a_Position, 1.0);
      }
  )";

  std::string fragment_source = R"(
      #version 330 core

      layout(location = 0) out vec4 color;

      in vec4 v_Color;

      void main() {
        color = v_Color;
      }
  )";


  shader_.reset(new renderer::Shader(vertex_source, fragment_source));
}

Application::~Application() {}

/**
 * This currently does a lot of custom rendering when in reality it should
 * be implemented by a child project that is running the game. This will change
 * in the future, but at the moment implements a lot of specific rendering
 * tests that are for ensuring that the renderer currently works.
 */
void Application::Run() {
  while (running_) {
    renderer::RenderCommand::SetClearColor({0.1f, 0.1f, 0.1f, 1.0f});
    renderer::RenderCommand::Clear();

    renderer::Renderer::BeginScene();

    renderer::Renderer::Submit(vertex_array_);
    shader_->Bind();

    renderer::Renderer::EndScene();

    for (Layer* layer : layer_stack_) {
      layer->OnUpdate();
    }

    imgui_layer_->Begin();
    for (Layer* layer : layer_stack_) {
      layer->OnImGuiRender();
    }
    imgui_layer_->End();

    window_->OnUpdate();
  }
}

/**
 * This function only specifically listens for when the window is requested to
 * close before passing the event to layers on the LayerStack.
 */
void Application::OnEvent(events::Event* event) {
  events::EventDispatcher dispatcher(event);
  dispatcher.Dispatch<events::WindowCloseEvent>
      (BIND_EVENT_FN(Application::OnWindowClosed));

  // Pass the event to all needed layers on the stack.
  for (auto it = layer_stack_.end(); it != layer_stack_.begin();) {
    (*--it)->OnEvent(event);
    if (event->HasBeenHandled()) {
      break;
    }
  }
}

void Application::PushLayer(Layer* layer) {
  layer_stack_.PushLayer(layer);
  layer->OnAttach();
}

void Application::PushOverlay(Layer* layer) {
  layer_stack_.PushOverlay(layer);
  layer->OnAttach();
}

bool Application::OnWindowClosed(const events::WindowCloseEvent& event) {
  running_ = false;
  return true;
}


}  // namespace engine
