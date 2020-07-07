#include <glm/glm.hpp>

#include "Engine.h"
#include "platform/opengl/OpenGLShader.h"
#include "ext/matrix_transform.hpp"
#include <imgui.h>
#include <glm/gtc/type_ptr.hpp>

class ExampleLayer : public engine::Layer {
 public:
  ExampleLayer() :
      Layer("Example"),
      camera_(-1.6f, 1.6f, -0.9f, 0.9f),
      camera_position_({0.0f, 0.0f, 0.0f}),
      square_position_(0.0f) {
    // Setup our vertices.
    float vertices[3 * 7] = {
      -0.5f, -0.5f, 0.0f, 0.0f, 0.0f, 0.9f, 1.0f,
       0.5f, -0.5f, 0.0f, 0.0f, 0.0f, 1.0f, 1.0f,
       0.0f, 0.5f, 0.0f, 1.0f, 1.0f, 0.9f, 1.0f,
    };

    vertex_array_.reset(engine::renderer::VertexArray::Create());

    vertex_buffer_ = engine::renderer::VertexBuffer::Create(
        vertices, sizeof(vertices));

    engine::renderer::BufferLayout layout_init_list = {
        { engine::renderer::ShaderDataType::Float3, "a_Position"},
        { engine::renderer::ShaderDataType::Float4, "a_Color", true}};

    engine::renderer::BufferLayout layout(layout_init_list);
    vertex_buffer_->SetLayout(layout);

    vertex_array_->AddVertexBuffer(vertex_buffer_);

    unsigned int indices[3] = { 0, 1, 2 };
    index_buffer_ = engine::renderer::IndexBuffer::Create(indices, 3);

    vertex_array_->SetIndexBuffer(index_buffer_);

    std::string vertex_source = R"(
        #version 330 core

        layout(location = 0) in vec3 a_Position;
        layout(location = 1) in vec4 a_Color;

        uniform mat4 u_ViewProjection;
        uniform mat4 u_Transform;

        out vec3 v_Position;
        out vec4 v_Color;

        void main() {
          v_Position = a_Position;
          v_Color = a_Color;
          gl_Position = u_ViewProjection * u_Transform * vec4(a_Position, 1.0);
        }
    )";

    std::string fragment_source = R"(
        #version 330 core

        layout(location = 0) out vec4 color;

        uniform mat4 u_ViewProjection;
        uniform vec4 u_Color;

        in vec4 v_Color;

        void main() {
          color = u_Color;
        }
    )";

    shader_.reset(
        engine::renderer::Shader::Create(vertex_source, fragment_source));
  }

  void OnUpdate(engine::util::TimeStep time_step) override {
    float ts = time_step.InSeconds<float>();
    float ts2 = time_step.InMicroSeconds<double>();

    if (engine::Input::IsKeyPressed(ENGINE_KEY_W)) {
      camera_position_.y += camera_speed_ * ts;
    } else if (engine::Input::IsKeyPressed(ENGINE_KEY_S)) {
      camera_position_.y -= camera_speed_ * ts;
    }

    if (engine::Input::IsKeyPressed(ENGINE_KEY_A)) {
      camera_position_.x -= camera_speed_ * ts;
    } else if (engine::Input::IsKeyPressed(ENGINE_KEY_D)) {
      camera_position_.x += camera_speed_ * ts;
    }

    if (engine::Input::IsKeyPressed(ENGINE_KEY_I)) {
      square_position_.y += square_move_speed_ * ts;
    } else if (engine::Input::IsKeyPressed(ENGINE_KEY_K)) {
      square_position_.y -= square_move_speed_ * ts;
    }

    if (engine::Input::IsKeyPressed(ENGINE_KEY_J)) {
      square_position_.x -= square_move_speed_ * ts;
    } else if (engine::Input::IsKeyPressed(ENGINE_KEY_L)) {
      square_position_.x += square_move_speed_ * ts;
    }

    engine::renderer::RenderCommand::SetClearColor({0.1f, 0.1f, 0.1f, 1.0f});
    engine::renderer::RenderCommand::Clear();

    camera_.SetPosition(camera_position_);
    camera_.SetRotation(45.0f);

    glm::mat4 scale = glm::scale(glm::mat4(1.0f), glm::vec3(0.1f));

    engine::renderer::Renderer::BeginScene(camera_);


    const auto& cast = std::dynamic_pointer_cast<
      engine::platform::opengl::OpenGLShader>(shader_);

    for (int y = 0; y < 20; ++y) {
      for (int x = 0; x < 20; ++x) {
        glm::vec3 pos(x * 0.11f, y * 0.11f, 0.0f);
        glm::mat4 transform = glm::translate(glm::mat4(1.0f), pos) * scale;

        if (x % 2 == 0) {
          cast->UploadUniformFloat4("u_Color", red_color_);
        } else {
          cast->UploadUniformFloat4("u_Color", blue_color_);
        }

        engine::renderer::Renderer::Submit(vertex_array_, shader_, transform);
      }
    }

    glm::mat4 transform = glm::translate(glm::mat4(1.0f), square_position_);
    engine::renderer::Renderer::Submit(vertex_array_, shader_, transform);
    engine::renderer::Renderer::EndScene();
  }

  void OnImGuiRender() override {
    ImGui::Begin("Settings");
    ImGui::ColorEdit4("Colors", glm::value_ptr(red_color_));
    ImGui::End();
  }

  void OnEvent(engine::events::Event* event) override {}

 private:
  std::shared_ptr<engine::renderer::Shader> shader_;
  std::shared_ptr<engine::renderer::VertexBuffer> vertex_buffer_;
  std::shared_ptr<engine::renderer::IndexBuffer> index_buffer_;
  std::shared_ptr<engine::renderer::VertexArray> vertex_array_;

  engine::renderer::OrthographicCamera camera_;
  glm::vec3 camera_position_;
  glm::vec3 square_position_;
  float camera_speed_ = 0.01f;
  float square_move_speed_ = 0.03f;
  glm::vec4 red_color_ = {0.8f, 0.3f, 0.2f, 1.0f};
  glm::vec4 blue_color_ = {0.2f, 0.3f, 0.8f, 1.0f};
};

class Sandbox : public engine::Application {
 public:
  Sandbox() { PushLayer(new ExampleLayer()); }
  ~Sandbox() {}
};

engine::Application* engine::CreateApplication() { return new Sandbox(); }
