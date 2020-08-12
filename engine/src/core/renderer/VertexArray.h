/**
 * @file VertexArray.h
 * @brief The Generic VertexArray API.
 *
 * Contains the generic API implementation for creating Vertex arrays that are
 * compatible with the engines rendering API.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_VERTEXARRAY_H_
#define ENGINE_SRC_CORE_RENDERER_VERTEXARRAY_H_

#include <memory>

#include "core/memory/Pointers.h"
#include "core/renderer/Buffer.h"

namespace engine {
namespace core {
namespace renderer {

class VertexArray {
 public:
  virtual ~VertexArray() {}

  virtual void Bind() const = 0;
  virtual void Unbind() const = 0;

  virtual void AddVertexBuffer(
      const memory::Shared<VertexBuffer>& vertex_buffer) = 0;

  virtual void SetIndexBuffer(
      const memory::Shared<IndexBuffer>& index_buffer) = 0;

  virtual const memory::Shared<IndexBuffer>&
      GetIndexBuffer() const = 0;

  virtual const std::vector<memory::Shared<VertexBuffer>>&
      GetVertexBuffers() const = 0;

  static memory::Shared<VertexArray> Create();
};

}  // namespace renderer
}  // namespace core
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_VERTEXARRAY_H_

/**
 * @class engine::renderer::VertexArray
 * @brief The abstraction for representing Vertex arrays and their sub
 * components.
 */

/**
 * @fn engine::renderer::VertexArray::Bind
 * @brief Bind the Vertex array and it's components to the rendering API and
 * GPU.
 */

/**
 * @fn engine::renderer::VertexArray::Unbind
 * @brief Unbind the vertex array and it's components from the rendering API
 * and memory.
 */

/**
 * @fn engine::renderer::VertexArray::AddVertexBuffer
 * @brief Add a vertex buffer to the vertex array.
 */

/**
 * @fn engine::renderer::VertexArray::SetIndexBuffer
 * @brief Set the index buffer of the vertex array.
 */

/**
 * @fn engine::renderer::VertexArray::GetIndexBuffers
 * @brief Get all the index buffers that are associated with this class.
 */

/**
 * @fn engine::renderer::VertexArray::Create
 * @brief Creates a vertex array through the platform specific API that is
 * being used at runtime.
 */
