#ifndef ENGINE_SRC_PLATFORM_OPENGL_OPENGLVERTEXARRAY_H_
#define ENGINE_SRC_PLATFORM_OPENGL_OPENGLVERTEXARRAY_H_

#include <bits/stdint-uintn.h>
#include <vector>

#include "core/memory/Pointers.h"
#include "core/renderer/Buffer.h"
#include "core/renderer/VertexArray.h"

namespace engine {
namespace platform {
namespace opengl {

/**
 * @class VertexArray
 * @brief The abstraction for representing Vertex arrays and their sub
 * components.
 */
class OpenGLVertexArray : public renderer::VertexArray {
 public:
  OpenGLVertexArray();
  ~OpenGLVertexArray();

  /**
   * @fn Bind
   * @brief Bind the Vertex array and it's components to the rendering API and
   * GPU.
   */
  void Bind() const override;

  /**
   * @fn Unbind
   * @brief Unbind the vertex array and it's components from the rendering API
   * and memory.
   */
  void Unbind() const override;

  /**
   * @fn AddVertexBuffer
   * @brief Add a vertex buffer to the current Vertex Array.
   */
  void AddVertexBuffer(
      const memory::Shared<renderer::VertexBuffer>& vertex_buffer) override;

  /**
   * @fn SetIndexBuffer
   * @brief Set the index buffer for rendering all vertex arrays.
   */
  void SetIndexBuffer(
      const memory::Shared<renderer::IndexBuffer>& index_buffer) override;

  /**
   * @fn GetIndexBuffer
   * @brief Get the index buffer associated with this Vertex Array.
   */
  inline const memory::Shared<renderer::IndexBuffer>& GetIndexBuffer()
      const override { return index_buffer_; }

  /**
   * @fn GetVertexBuffers
   * @brief Get the Vertex Buffers that are associated with this Vertex Array.
   */
  inline const std::vector<memory::Shared<renderer::VertexBuffer>>&
     GetVertexBuffers() const override { return vertex_buffers_; }

 private:
  memory::Shared<renderer::IndexBuffer> index_buffer_;
  std::vector<memory::Shared<renderer::VertexBuffer>> vertex_buffers_;
  uint32_t renderer_id_;

};


}  // namespace opengl
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_SRC_PLATFORM_OPENGL_OPENGLVERTEXARRAY_H_
