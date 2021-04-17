#include "Lambda/core/renderer/Renderer2D.h"

#include <glm/gtc/matrix_transform.hpp>

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/RenderCommand.h"
#include "Lambda/core/renderer/Shader.h"
#include "Lambda/core/renderer/VertexArray.h"
#include "Lambda/lib/Assert.h"

namespace lambda {
namespace core {
namespace renderer {

namespace {

/// @brief Internal storage for the 2D rendering API. It is not yet finalized.
struct Renderer2DStorage {
  memory::Shared<VertexArray> QuadVertexArray;
  memory::Shared<Shader> TextureShader;
  memory::Shared<Texture2D> WhiteTexture;
};

/// @brief A static instance of the renderers storage.
static memory::Unique<Renderer2DStorage> kRendererStorage;

}  // namespace

/// @todo (C3NZ): This is currently dependent on opengl but implemented within
/// the engines abstraction layer.
void Renderer2D::Init() {
  kRendererStorage = memory::CreateUnique<Renderer2DStorage>();

  float vertices[5 * 4] = {
    -0.5f, -0.5f, 0.0f, 0.0f, 0.0f,
     0.5f, -0.5f, 0.0f, 1.0f, 0.0f,
     0.5f,  0.5f, 0.0f, 1.0f, 1.0f,
    -0.5f,  0.5f, 0.0f, 0.0f, 1.0f
  };

  kRendererStorage->QuadVertexArray = VertexArray::Create();

  memory::Shared<VertexBuffer> vertex_buffer = VertexBuffer::Create(
      vertices, sizeof(vertices));

  BufferLayout layout_init_list = {
    { ShaderDataType::Float3, "a_Position" },
    { ShaderDataType::Float2, "a_TexCoord" }};

  BufferLayout layout(layout_init_list);
  vertex_buffer->SetLayout(layout);

  kRendererStorage->QuadVertexArray->AddVertexBuffer(vertex_buffer);

  uint32_t indices[6] = { 0, 1, 2, 2, 3, 0 };
  memory::Shared<IndexBuffer> index_buffer = IndexBuffer::Create(
      indices, sizeof(indices) / sizeof(uint32_t));

  kRendererStorage->QuadVertexArray->SetIndexBuffer(index_buffer);

  /// Create a simple and small white texture.
  kRendererStorage->WhiteTexture = Texture2D::Create(1, 1);
  uint32_t white_texture_data = 0xffffffff;
  kRendererStorage->WhiteTexture->SetData(
      &white_texture_data, sizeof(uint32_t));

  // Create and bind our basic shader.
  kRendererStorage->TextureShader = Shader::Create(
      "assets/shaders/Texture.glsl");
  kRendererStorage->TextureShader->Bind();
  kRendererStorage->TextureShader->SetInt("u_Texture", 0);
}

/// This will completely reset all of the memory owned by the the renderers
/// storage system. In the future, the memory allocator should ensure that
/// resources are freed once the renderers storage has been released.
void Renderer2D::Shutdown() {
  kRendererStorage->QuadVertexArray.reset();
  kRendererStorage->TextureShader.reset();

  kRendererStorage.reset();
}

/// @todo (C3NZ): This needs to be altered to not be dependent on OpenGL code
/// and instead be implemented within the platform API.
void Renderer2D::BeginScene(const OrthographicCamera& camera) {
  kRendererStorage->TextureShader->Bind();

  kRendererStorage->TextureShader->SetMat4(
      "u_ViewProjection", camera.GetViewProjectionMatrix());
}

void Renderer2D::EndScene() {}

/// Used for drawing quads that are on the surface of the screen.
/// Automatically forwards your arguments into the other DrawQuad overload with
/// position being modified to be a vec3 with a z of 0.
void Renderer2D::DrawQuad(
    const glm::vec2& position,
    const glm::vec2& size,
    const glm::vec4& color) {
  DrawQuad({position.x, position.y, 0.0f}, size, color);
}

void Renderer2D::DrawQuad(
    const glm::vec3& position,
    const glm::vec2& size,
    const glm::vec4& color) {
  kRendererStorage->TextureShader->SetFloat4("u_Color", color);
  kRendererStorage->WhiteTexture->Bind();

  // Translation, times rotation, times scale. (Must be in that order,
  // since matrix multiplication has an effect on the output.)
  // This allows the size to be set externally.
  glm::mat4 transform = glm::translate(
      glm::mat4(1.0f), position) * glm::scale(
          glm::mat4(1.0f), {size.x, size.y, 1.0f});
  kRendererStorage->TextureShader->SetMat4("u_Transform", transform);

  // Bind vertices, draw them, and then unbind the texture.
  kRendererStorage->QuadVertexArray->Bind();
  RenderCommand::DrawIndexed(kRendererStorage->QuadVertexArray);
  kRendererStorage->WhiteTexture->Unbind();
}

void Renderer2D::DrawQuad(
    const glm::vec2& position,
    const glm::vec2& size,
    memory::Shared<Texture2D> texture) {
  DrawQuad({position.x, position.y, 0.0f}, size, texture);
}

void Renderer2D::DrawQuad(
    const glm::vec3& position,
    const glm::vec2& size,
    memory::Shared<Texture2D> texture) {
  kRendererStorage->TextureShader->SetFloat4("u_Color", glm::vec4(1.0f));
  texture->Bind();

  // Translation, times rotation, times scale. (Must be in that order,
  // since matrix multiplication has an effect on the output.)
  // This allows the size to be set externally.
  glm::mat4 transform = glm::translate(
      glm::mat4(1.0f), position) * glm::scale(
          glm::mat4(1.0f), {size.x, size.y, 1.0f});
  kRendererStorage->TextureShader->SetMat4("u_Transform", transform);

  // Bind vertices, draw them, and then unbind the texture.
  kRendererStorage->QuadVertexArray->Bind();
  RenderCommand::DrawIndexed(kRendererStorage->QuadVertexArray);
  texture->Unbind();
}

}  // namespace renderer
}  // namespace core
}  // namespace lambda
