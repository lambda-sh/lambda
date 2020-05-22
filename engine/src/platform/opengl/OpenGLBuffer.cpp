#include "platform/opengl/OpenGLBuffer.h"

#include <glad/glad.h>

namespace engine {
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

OpenGLIndexBuffer::OpenGLIndexBuffer(uint32_t* indices, uint32_t count) : count_(count) {
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
}  // namespace engine
