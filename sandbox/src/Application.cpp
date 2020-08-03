#include <thread>

#include <glm/glm.hpp>
#include <glm/gtc/type_ptr.hpp>
#include <imgui.h>

#include "Engine.h"
#include "ext/matrix_transform.hpp"
#include "platform/opengl/OpenGLShader.h"

using engine::events::Event;
using engine::layers::Layer;
using engine::memory::Shared;
using engine::renderer::BufferLayout;
using engine::renderer::IndexBuffer;
using engine::renderer::OrthographicCamera;
using engine::renderer::RenderCommand;
using engine::renderer::Renderer;
using engine::renderer::Shader;
using engine::renderer::ShaderDataType;
using engine::renderer::ShaderDataType;
using engine::renderer::ShaderLibrary;
using engine::renderer::Texture2D;
using engine::renderer::VertexArray;
using engine::renderer::VertexBuffer;
using engine::util::TimeStep;

class ExampleLayer : public Layer {
 public:
  ExampleLayer() :
      Layer("Example"),
      camera_(-1.6f, 1.6f, -0.9f, 0.9f),
      camera_position_({0.0f, 0.0f, 0.0f}),
      square_position_(0.0f) {
    // Initialize the renderer. (Handles graphics specific API setup.)
    loop_ = engine::memory::CreateShared<engine::io::EventLoop>();

    loop_->SetInterval(
        [&]() {
          ENGINE_CLIENT_INFO("Executing every 2 seconds!");
          return true;
          }, 2000);

    std::thread t([&]() { loop_->Run(); });
    t.detach();
    Renderer::Init();

    float vertices[3 * 7] = {
      -0.5f, -0.5f, 0.0f, 0.0f, 0.0f,
       0.5f, -0.5f, 0.0f, 1.0f, 0.0f,
       0.0f,  0.5f, 0.0f, 0.0f, 1.0f};

    vertex_array_ = VertexArray::Create();

    vertex_buffer_ = VertexBuffer::Create(
        vertices, sizeof(vertices));

    BufferLayout layout_init_list = {
        { ShaderDataType::Float3, "a_Position"},
        { ShaderDataType::Float2, "a_TexCoord"}};

    BufferLayout layout(layout_init_list);
    vertex_buffer_->SetLayout(layout);

    vertex_array_->AddVertexBuffer(vertex_buffer_);

    unsigned int indices[3] = { 0, 1, 2 };
    index_buffer_ = IndexBuffer::Create(indices, 3);

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
        in vec2 v_TexCoord;

        void main() {
          color = u_Color;
        }
    )";

    shader_lib_.Add(
        Shader::Create(
            "yeet", vertex_source, fragment_source));

    shader_lib_.Add(
        Shader::Create("assets/shaders/Texture.glsl"));

    shader_lib_.Load("Texture2", "assets/shaders/Texture.glsl");

    texture_ = Texture2D::Create(
        "assets/textures/checkboard.png");

    lambda_texture_ = Texture2D::Create(
        "assets/textures/hl2.png");

    std::dynamic_pointer_cast< engine::platform::opengl::OpenGLShader>(
        shader_lib_.Get("yeet"))->Bind();
    std::dynamic_pointer_cast<engine::platform::opengl::OpenGLShader>(
        shader_lib_.Get("yeet"))->UploadUniformInt("u_Texture", 0);
  }

  void OnUpdate(TimeStep time_step) override {
    // Demo of the ability to get the time in many different time precisions
    // with different floating point precisions.
    float ts = time_step.InSeconds<float>();
    float ts2 = time_step.InMicroSeconds<double>();

    using engine::Input;

    if (Input::IsKeyPressed(ENGINE_KEY_W)) {
      camera_position_.y += camera_speed_ * ts;
    } else if (Input::IsKeyPressed(ENGINE_KEY_S)) {
      camera_position_.y -= camera_speed_ * ts;
    }

    if (Input::IsKeyPressed(ENGINE_KEY_A)) {
      camera_position_.x -= camera_speed_ * ts;
    } else if (Input::IsKeyPressed(ENGINE_KEY_D)) {
      camera_position_.x += camera_speed_ * ts;
    }

    if (Input::IsKeyPressed(ENGINE_KEY_I)) {
      square_position_.y += square_move_speed_ * ts;
    } else if (Input::IsKeyPressed(ENGINE_KEY_K)) {
      square_position_.y -= square_move_speed_ * ts;
    }

    if (Input::IsKeyPressed(ENGINE_KEY_J)) {
      square_position_.x -= square_move_speed_ * ts;
    } else if (Input::IsKeyPressed(ENGINE_KEY_L)) {
      square_position_.x += square_move_speed_ * ts;
    }

    RenderCommand::SetClearColor({0.1f, 0.1f, 0.1f, 1.0f});
    RenderCommand::Clear();

    camera_.SetPosition(camera_position_);
    camera_.SetRotation(45.0f);

    glm::mat4 scale = glm::scale(glm::mat4(1.0f), glm::vec3(0.1f));

    Renderer::BeginScene(camera_);


    const auto& cast = std::dynamic_pointer_cast<
      engine::platform::opengl::OpenGLShader>(shader_lib_.Get("yeet"));

    for (int y = 0; y < 20; ++y) {
      for (int x = 0; x < 20; ++x) {
        glm::vec3 pos(x * 0.11f, y * 0.11f, 0.0f);
        glm::mat4 transform = glm::translate(glm::mat4(1.0f), pos) * scale;

        if (x % 2 == 0) {
          cast->UploadUniformFloat4("u_Color", red_color_);
        } else {
          cast->UploadUniformFloat4("u_Color", blue_color_);
        }

        Renderer::Submit(
            vertex_array_, shader_lib_.Get("yeet"), transform);
      }
    }

    glm::mat4 transform = glm::translate(glm::mat4(1.0f), square_position_);

    // Bind the texture before using it with the shader.
    texture_->Bind();
    Renderer::Submit(
        vertex_array_, shader_lib_.Get("Texture"), transform);

    lambda_texture_->Bind();
    Renderer::Submit(
        vertex_array_, shader_lib_.Get("Texture"), transform);

    Renderer::EndScene();
  }

  void OnImGuiRender() override {
    ImGui::Begin("Settings");
    ImGui::ColorEdit4("Colors", glm::value_ptr(red_color_));
    ImGui::End();
  }

  void OnEvent(Shared<Event> event) override {}

 private:
  Shared<VertexBuffer> vertex_buffer_;
  Shared<IndexBuffer> index_buffer_;
  Shared<VertexArray> vertex_array_;
  Shared<Texture2D> texture_, lambda_texture_;
  Shared<engine::io::EventLoop> loop_;
  ShaderLibrary shader_lib_;

  OrthographicCamera camera_;
  glm::vec3 camera_position_;
  glm::vec3 square_position_;
  float camera_speed_ = 0.01f;
  float square_move_speed_ = 0.03f;
  glm::vec4 red_color_ = {0.8f, 0.3f, 0.2f, 1.0f};
  glm::vec4 blue_color_ = {0.2f, 0.3f, 0.8f, 1.0f};
};

class Sandbox : public engine::Application {
 public:
  Sandbox() {
    PushLayer(engine::memory::CreateShared<ExampleLayer>());
  }
  ~Sandbox() {}
};

engine::Application* engine::CreateApplication() { return new Sandbox(); }
