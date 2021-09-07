/// @file OpenGLVertexArray.h
/// @brief OpenGL implementations of lambda vertex arrays.
#ifndef LAMBDA_PLATFORM_OPENGL_OPENGLVERTEXARRAY_H_
#define LAMBDA_PLATFORM_OPENGL_OPENGLVERTEXARRAY_H_

#include <vector>

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/Buffer.h"
#include "Lambda/core/renderer/VertexArray.h"

namespace lambda {
namespace platform {
namespace opengl {

/// @brief The abstraction for representing Vertex arrays and their sub
/// components.
class OpenGLVertexArray : public core::renderer::VertexArray {
 public:
  OpenGLVertexArray();
  ~OpenGLVertexArray();

  /// @brief Bind the Vertex array and it's components to the rendering API and
  /// GPU.
  void Bind() const override;

  /// @brief Unbind the vertex array and it's components from the rendering API
  /// and memory.
  void Unbind() const override;

 /// @brief Add a vertex buffer to the current Vertex Array.
  void AddVertexBuffer(
      const core::memory::Shared<
          core::renderer::VertexBuffer>& vertex_buffer) override;

  /// @brief Set the index buffer for rendering all vertex arrays.
  void SetIndexBuffer(
      const core::memory::Shared<core::renderer::IndexBuffer>& index_buffer)
          override;

  /// @brief Get the index buffer associated with this Vertex Array.
  const core::memory::Shared<core::renderer::IndexBuffer> GetIndexBuffer()
      const override { return index_buffer_; }

  /// @brief Get the Vertex Buffers that are associated with this Vertex Array.
  const std::vector<core::memory::Shared<core::renderer::VertexBuffer>>
     GetVertexBuffers() const override { return vertex_buffers_; }

 private:
  uint32_t renderer_id_;
  core::memory::Shared<core::renderer::IndexBuffer> index_buffer_;
  std::vector<core::memory::Shared<core::renderer::VertexBuffer>>
      vertex_buffers_;

};

}  // namespace opengl
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_OPENGL_OPENGLVERTEXARRAY_H_
