#include "Engine.h"

class ExampleLayer : public engine::Layer {
 public:
  ExampleLayer() : Layer("Example"), camera_(-1.6f, 1.6f, -0.9f, 0.9f) {
    // Setup our vertices.
    float vertices[3 * 7] = {
      -0.5f, -0.5f, 0.0f, 0.0f, 0.0f, 0.9f, 1.0f,
       0.5f, -0.5f, 0.0f, 0.0f, 0.0f, 1.0f, 1.0f,
       0.0f, 0.5f, 0.0f, 1.0f, 1.0f, 0.9f, 1.0f,
    };

    vertex_array_.reset(engine::renderer::VertexArray::Create());

    vertex_buffer_.reset(
        engine::renderer::VertexBuffer::Create(vertices, sizeof(vertices)));

    engine::renderer::BufferLayout layout_init_list = {
        { engine::renderer::ShaderDataType::Float3, "a_Position"},
        { engine::renderer::ShaderDataType::Float4, "a_Color", true}};

    engine::renderer::BufferLayout layout(layout_init_list);
    vertex_buffer_->SetLayout(layout);

    vertex_array_->AddVertexBuffer(vertex_buffer_);

    unsigned int indices[3] = { 0, 1, 2 };
    index_buffer_.reset(engine::renderer::IndexBuffer::Create(indices, 3));

    vertex_array_->SetIndexBuffer(index_buffer_);

    std::string vertex_source = R"(
        #version 330 core

        layout(location = 0) in vec3 a_Position;
        layout(location = 1) in vec4 a_Color;

        uniform mat4 u_ViewProjection;

        out vec3 v_Position;
        out vec4 v_Color;

        void main() {
          v_Position = a_Position;
          v_Color = a_Color;
          gl_Position = u_ViewProjection * vec4(a_Position, 1.0);
        }
    )";

    std::string fragment_source = R"(
        #version 330 core

        layout(location = 0) out vec4 color;

        uniform mat4 u_ViewProjection;

        in vec4 v_Color;

        void main() {
          color = v_Color;
        }
    )";

    shader_.reset(new engine::renderer::Shader(vertex_source, fragment_source));
  }

  void OnUpdate() override {
    engine::renderer::RenderCommand::SetClearColor({0.1f, 0.1f, 0.1f, 1.0f});
    engine::renderer::RenderCommand::Clear();

    camera_.SetPosition({0.5f, 0.5f, 0.0f});
    camera_.SetRotation(45.0f);

    engine::renderer::Renderer::BeginScene(camera_);
    engine::renderer::Renderer::Submit(vertex_array_, shader_);
    engine::renderer::Renderer::EndScene();
  }

  void OnImGuiRender() override {}

  void OnEvent(engine::events::Event* event) override {}

 private:
  std::shared_ptr<engine::renderer::Shader> shader_;
  std::shared_ptr<engine::renderer::VertexBuffer> vertex_buffer_;
  std::shared_ptr<engine::renderer::IndexBuffer> index_buffer_;
  std::shared_ptr<engine::renderer::VertexArray> vertex_array_;

  engine::renderer::OrthographicCamera camera_;
};

class Sandbox : public engine::Application {
 public:
  Sandbox() {
    PushLayer(new ExampleLayer());
  }

  ~Sandbox() {}
};

engine::Application* engine::CreateApplication() { return new Sandbox(); }
