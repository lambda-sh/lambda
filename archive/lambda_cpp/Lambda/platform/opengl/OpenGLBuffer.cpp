#include "Lambda/platform/opengl/OpenGLBuffer.h"

#include <Lambda/platform/glad/Glad.h>

namespace lambda {
namespace platform {
namespace opengl {

// ----------------------------- VERTEX BUFFER IMPL ----------------------------

OpenGLVertexBuffer::OpenGLVertexBuffer(float* vertices, uint32_t size) {
  glCreateBuffers(1, &renderer_ID_);
  glBindBuffer(GL_ARRAY_BUFFER, renderer_ID_);
  glBufferData(GL_ARRAY_BUFFER, size, vertices, GL_STATIC_DRAW);
}

OpenGLVertexBuffer::~OpenGLVertexBuffer() {
  glDeleteBuffers(1, &renderer_ID_);
}

void OpenGLVertexBuffer::Bind() const {
  glBindBuffer(GL_ARRAY_BUFFER, renderer_ID_);
}

void OpenGLVertexBuffer::Unbind() const {
  glBindBuffer(GL_ARRAY_BUFFER, 0);
}

// ----------------------------- INDEX BUFFER IMPL ----------------------------

/// Constructs an Index buffer given an array of indices and total count. The
/// instantiation of the index buffer will cause the engine to to assert an
/// error if the count is > 0.
OpenGLIndexBuffer::OpenGLIndexBuffer(uint32_t* indices, uint32_t count)
    : count_(count) {
  LAMBDA_CORE_ASSERT(
      count > 0,
      "There must be more than 0 indices in order to create an index buffer",
      "");

  glCreateBuffers(1, &renderer_ID_);
  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, renderer_ID_);
  glBufferData(
      GL_ELEMENT_ARRAY_BUFFER,
      count * sizeof(uint32_t),
      indices,
      GL_STATIC_DRAW);
}

OpenGLIndexBuffer::~OpenGLIndexBuffer() {
  glDeleteBuffers(1, &renderer_ID_);
}

void OpenGLIndexBuffer::Bind() const {
  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, renderer_ID_);
}

void OpenGLIndexBuffer::Unbind() const {
  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);
}

}  // namespace renderer
}  // namespace platform
}  // namespace lambda
