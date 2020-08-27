#include "Lambda/core/renderer/Renderer2D.h"

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/Shader.h"
#include "Lambda/core/renderer/VertexArray.h"

namespace lambda {
namespace core {
namespace renderer {

namespace {

struct Renderer2DStorage {
  core::memory::Shared<VertexArray> QuadVertexArray;
  core::memory::Shared<Shader> FlatColorShader;
};

static core::memory::Shared<Renderer2DStorage> kRendererStorage;

}  // namespace


/// This is currently dependent on opengl
void Renderer2D::Init() {
  kRendererStorage.reset(new Renderer2DStorage());

  float vertices[3 * 3] = {
    -0.5f, -0.5f, 0.0f,
     0.5f, -0.5f, 0.0f,
     0.0f,  0.5f, 0.0};

  kRendererStorage->QuadVertexArray = VertexArray::Create();

  core::memory::Shared<VertexBuffer> vertex_buffer = VertexBuffer::Create(
      vertices, sizeof(vertices));

  BufferLayout layout_init_list = {{ ShaderDataType::Float3, "a_Position" }};
  BufferLayout layout(layout_init_list);
  vertex_buffer->SetLayout(layout);

  kRendererStorage->QuadVertexArray->AddVertexBuffer(vertex_buffer);

  unsigned int indices[3] = { 0, 1, 2 };
  core::memory::Shared<IndexBuffer> index_buffer = IndexBuffer::Create(
      indices, 3);

  kRendererStorage->QuadVertexArray->SetIndexBuffer(index_buffer);
  kRendererStorage->FlatColorShader = core::renderer::Shader::Create(
      "assets/shaders/FlatColor.glsl");
}

void Renderer2D::Shutdown() {}

void Renderer2D::BeginScene(const OrthographicCamera& camera) {}

void Renderer2D::EndScene() {}

void Renderer2D::DrawQuad(
    const glm::vec2& position,
    const glm::vec2& size,
    const glm::vec4& color) {}

void Renderer2D::DrawQuad(
    const glm::vec3& position,
    const glm::vec2& size,
    const glm::vec4& color) {}

}  // namespace renderer
}  // namespace core
}  // namespace lambda
