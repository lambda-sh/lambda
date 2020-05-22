#include "core/Application.h"

#include <functional>
#include <memory>

#include <glad/glad.h>

#include "core/Assert.h"
#include "core/Input.h"
#include "core/Layer.h"
#include "core/Log.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"

#include "core/renderer/Shader.h"

namespace engine {

Application* Application::kApplication_ = nullptr;

Application::Application() {
  ENGINE_CORE_ASSERT(!kApplication_, "Application already exists.");
  kApplication_ = this;

  window_ = std::unique_ptr<Window>(Window::Create());
  window_->SetEventCallback(BIND_EVENT_FN(Application::OnEvent));

  imgui_layer_ = new imgui::ImGuiLayer();
  PushLayer(imgui_layer_);

  // Generate and bind the vertex array.
  glGenVertexArrays(1, &vertex_array_);
  glBindVertexArray(vertex_array_);

  // Setup our vertices.
  float vertices[3 * 3] = {
    -0.5f, -0.5f, 0.0f,
     0.5f, -0.5f, 0.0f,
     0.0f,  0.5f, 0.0f,
  };

  // Setup the vertex buffer and bind it.
  vertex_buffer_.reset(
      renderer::VertexBuffer::Create(vertices, sizeof(vertices)));
  vertex_buffer_->Bind();

  // Enable the vertex attribute array and then define our vertex attributes.
  glEnableVertexAttribArray(0);
  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), nullptr);

  // Setup our indices and draw them to the screen.
  unsigned int indices[3] = { 0, 1, 2 };
  index_buffer_.reset(renderer::IndexBuffer::Create(indices, 3));
  index_buffer_->Bind();

  std::string vertex_source = R"(
      #version 330 core

      layout(location = 0) in vec3 position_attribute;

      void main() {
        gl_Position = vec4(position_attribute, 1.0);
      }
  )";

  std::string fragment_source = R"(
      #version 330 core

      layout(location = 0) out vec4 color;

      void main() {
        color = vec4(0.3, 0.5, 0.3, 1.0);
      }
  )";


  shader_.reset(new renderer::Shader(vertex_source, fragment_source));
}

Application::~Application() {}

// Specifically retrieve updates from the window first to dispatch input
// events to every layer before making updates to them.
// TODO(C3NZ): Check to see which kind of updates need to come first and what
// the performance impact of each are.
void Application::Run() {
  while (running_) {
    glClearColor(0.2f, 0.2f, 0.2f, 1);
    glClear(GL_COLOR_BUFFER_BIT);

    shader_->Bind();

    // Bind the vertex array and then draw all of it's elements.
    glBindVertexArray(vertex_array_);
    glDrawElements(
        GL_TRIANGLES, index_buffer_->GetCount(), GL_UNSIGNED_INT, nullptr);

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

// This is the primary handler for passing events generated from the window back
// into the our application and game.
void Application::OnEvent(events::Event* event) {
  events::EventDispatcher dispatcher(event);
  dispatcher.Dispatch<events::WindowCloseEvent>
      (BIND_EVENT_FN(Application::OnWindowClosed));
  ENGINE_CORE_TRACE(*event);

  // Pass the event to all needed layers on the stack.
  for (auto it = layer_stack_.end(); it != layer_stack_.begin();) {
    (*--it)->OnEvent(event);
    if (event->HasBeenHandled()) {
      break;
    }
  }
}

}  // namespace engine
