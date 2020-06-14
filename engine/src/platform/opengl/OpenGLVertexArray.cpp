#include "platform/opengl/OpenGLVertexArray.h"

#include <glad/glad.h>

#include "core/util/Assert.h"
#include "core/renderer/Buffer.h"

namespace engine {
namespace platform {
namespace opengl {

/**
 * @fn ShaderDataTypeToOpenGLBaseType
 * @brief Convert an renderer shader type to it's corresponding OpenGL type.
 */
static GLenum ShaderDataTypeToOpenGLBaseType(
    engine::renderer::ShaderDataType type) {
  switch (type) {
    case engine::renderer::ShaderDataType::Bool: return GL_BOOL;
    case engine::renderer::ShaderDataType::Float2: return GL_FLOAT;
    case engine::renderer::ShaderDataType::Float3: return GL_FLOAT;
    case engine::renderer::ShaderDataType::Float4: return GL_FLOAT;
    case engine::renderer::ShaderDataType::Float: return GL_FLOAT;
    case engine::renderer::ShaderDataType::Int2: return GL_INT;
    case engine::renderer::ShaderDataType::Int3: return GL_INT;
    case engine::renderer::ShaderDataType::Int4: return GL_INT;
    case engine::renderer::ShaderDataType::Int: return GL_INT ;
    case engine::renderer::ShaderDataType::Mat3: return GL_FLOAT;
    case engine::renderer::ShaderDataType::Mat4: return GL_FLOAT;
    default: ENGINE_CORE_ASSERT(false, "Unknown shader type."); return 0;
  }
}

OpenGLVertexArray::OpenGLVertexArray() {
  glCreateVertexArrays(1, &renderer_id_);
}

OpenGLVertexArray::~OpenGLVertexArray() {
  glDeleteVertexArrays(1, &renderer_id_);
}

void OpenGLVertexArray::Bind() const {
  glBindVertexArray(renderer_id_);
}

void OpenGLVertexArray::Unbind() const {
  glBindVertexArray(0);
}

void OpenGLVertexArray::AddVertexBuffer(
    const std::shared_ptr<renderer::VertexBuffer>& vertex_buffer) {
  glBindVertexArray(renderer_id_);
  vertex_buffer->Bind();


  uint32_t index = 0;
  const engine::renderer::BufferLayout& layout = vertex_buffer->GetLayout();
  ENGINE_CORE_ASSERT(layout.HasElements(), "The vertex buffer doesn't have a layout.");

  for (const renderer::BufferElement& element : layout) {
    glEnableVertexAttribArray(index);
    glVertexAttribPointer(
        index,
        element.Components,
        ShaderDataTypeToOpenGLBaseType(element.Type),
        element.Normalized ? GL_TRUE : GL_FALSE,
        layout.GetStride(),
        reinterpret_cast<const void*>(element.Offset));
    ++index;
  }

  vertex_buffers_.push_back(vertex_buffer);
}

void OpenGLVertexArray::SetIndexBuffer(
    const std::shared_ptr<renderer::IndexBuffer>& index_buffer) {
  glBindVertexArray(renderer_id_);
  index_buffer->Bind();

  index_buffer_ = index_buffer;
}

}  // namespace opengl
}  // namespace platform
}  // namespace engine
