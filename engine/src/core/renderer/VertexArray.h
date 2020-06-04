/**
 * @file engine/src/core/renderer/VertexArray.h
 * @brief The Generic VertexArray API.
 *
 * Contains the generic API implementation for creating Vertex arrays that are
 * compatible with the engines rendering API.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_VERTEXARRAY_H_
#define ENGINE_SRC_CORE_RENDERER_VERTEXARRAY_H_

#include <memory>

#include "core/renderer/Buffer.h"

namespace engine {
namespace renderer {

/**
 * @class VertexArray
 * @brief The abstraction for representing Vertex arrays and their sub
 * components.
 */
class VertexArray {
 public:
  virtual ~VertexArray() {}

  /**
   * @fn Bind
   * @brief Bind the Vertex array and it's components to the rendering API and
   * GPU.
   */
  virtual void Bind() const = 0;

  /**
   * @fn Unbind
   * @brief Unbind the vertex array and it's components from the rendering API
   * and memory.
   */
  virtual void Unbind() const = 0;

  virtual void AddVertexBuffer(
      const std::shared_ptr<VertexBuffer>& vertex_buffer) = 0;

  virtual void SetIndexBuffer(
      const std::shared_ptr<IndexBuffer>& index_buffer) = 0;

  static VertexArray* Create();
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_VERTEXARRAY_H_
