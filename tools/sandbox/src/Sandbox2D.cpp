#include "Sandbox2D.h"

#include <Lambda.h>
#include <glm/glm.hpp>
#include <glm/gtc/type_ptr.hpp>
#include <imgui.h>

/// @todo (C3NZ): Rewrite the shader implementation to not have to rely on the
/// use of platform specific graphic APIs.
#include "Lambda/platform/opengl/OpenGLShader.h"

using lambda::core::renderer::BufferLayout;
using lambda::core::renderer::IndexBuffer;
using lambda::core::renderer::ShaderDataType;
using lambda::core::renderer::VertexArray;
using lambda::core::renderer::VertexBuffer;

namespace renderer = lambda::core::renderer;

namespace tools {
namespace sandbox {

void Sandbox2D::OnAttach() {
  renderer::Renderer2D::Init();

  float vertices[3 * 3] = {
    -0.5f, -0.5f, 0.0f,
     0.5f, -0.5f, 0.0f,
     0.0f,  0.5f, 0.0};

  vertex_array_ = VertexArray::Create();

  vertex_buffer_ = VertexBuffer::Create(
      vertices, sizeof(vertices));

  BufferLayout layout_init_list = {{ ShaderDataType::Float3, "a_Position" }};
  BufferLayout layout(layout_init_list);
  vertex_buffer_->SetLayout(layout);

  vertex_array_->AddVertexBuffer(vertex_buffer_);

  unsigned int indices[3] = { 0, 1, 2 };
  index_buffer_ = IndexBuffer::Create(indices, 3);

  vertex_array_->SetIndexBuffer(index_buffer_);
  shader_ = lambda::core::renderer::Shader::Create(
      "assets/shaders/FlatColor.glsl");
}

void Sandbox2D::OnDetach() {
  shader_->Unbind();
}

void Sandbox2D::OnUpdate(lambda::core::util::TimeStep delta) {
  camera_controller_.OnUpdate(delta);

  renderer::RenderCommand::SetClearColor({ 0.1f, 0.1f, 0.1f, 1.0f });
  renderer::RenderCommand::Clear();

  renderer::Renderer2D::BeginScene(camera_controller_.GetOrthographicCamera());

  std::dynamic_pointer_cast<lambda::platform::opengl::OpenGLShader>(
      shader_)->Bind();
  std::dynamic_pointer_cast<lambda::platform::opengl::OpenGLShader>(
      shader_)->UploadUniformFloat4("u_Color", shader_color_);

  renderer::Renderer::Submit(
      vertex_array_, shader_, glm::scale(glm::mat4(1.0f), glm::vec3(1.5f)));

  renderer::Renderer2D::DrawQuad(
      {0.0f, 0.0f}, {1.0f, 1.0f}, {0.8f, 0.2f, 0.3f, 1.0f});

  renderer::Renderer2D::EndScene();
}

void Sandbox2D::OnImGuiRender() {
  ImGui::Begin("Settings");
  ImGui::ColorEdit3("Shader color", glm::value_ptr(shader_color_));
  ImGui::End();
}

void Sandbox2D::OnEvent(
    lambda::core::memory::Shared<lambda::core::events::Event> event) {
  camera_controller_.OnEvent(event);
}

}  // namespace sandbox
}  // namespace tools
