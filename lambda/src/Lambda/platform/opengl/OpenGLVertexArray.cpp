#include "Lambda/platform/opengl/OpenGLVertexArray.h"

#include <Lambda/platform/glad/Glad.h>

#include "Lambda/lib/Assert.h"
#include "Lambda/core/renderer/Buffer.h"

namespace lambda {
namespace platform {
namespace opengl {

/**
 * @fn ShaderDataTypeToOpenGLBaseType
 * @brief Convert an renderer shader type to it's corresponding OpenGL type.
 */
static GLenum ShaderDataTypeToOpenGLBaseType(
    core::renderer::ShaderDataType type) {
  switch (type) {
    case core::renderer::ShaderDataType::Bool: return GL_BOOL;
    case core::renderer::ShaderDataType::Float2: return GL_FLOAT;
    case core::renderer::ShaderDataType::Float3: return GL_FLOAT;
    case core::renderer::ShaderDataType::Float4: return GL_FLOAT;
    case core::renderer::ShaderDataType::Float: return GL_FLOAT;
    case core::renderer::ShaderDataType::Int2: return GL_INT;
    case core::renderer::ShaderDataType::Int3: return GL_INT;
    case core::renderer::ShaderDataType::Int4: return GL_INT;
    case core::renderer::ShaderDataType::Int: return GL_INT ;
    case core::renderer::ShaderDataType::Mat3: return GL_FLOAT;
    case core::renderer::ShaderDataType::Mat4: return GL_FLOAT;
    default: LAMBDA_CORE_ASSERT(false, "Unknown shader type.", ""); return 0;
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
    const core::memory::Shared<core::renderer::VertexBuffer>& vertex_buffer) {
  glBindVertexArray(renderer_id_);
  vertex_buffer->Bind();


  uint32_t index = 0;
  const core::renderer::BufferLayout& layout = vertex_buffer->GetLayout();
  LAMBDA_CORE_ASSERT(layout.HasElements(), "The vertex buffer doesn't have a layout.", "");

  for (const core::renderer::BufferElement& element : layout) {
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
    const core::memory::Shared<core::renderer::IndexBuffer>& index_buffer) {
  glBindVertexArray(renderer_id_);
  index_buffer->Bind();

  index_buffer_ = index_buffer;
}

}  // namespace opengl
}  // namespace platform
}  // namespace lambda
