/// @file VertexArray.h
/// @brief The Generic VertexArray API.
///
/// Contains the generic API implementation for creating Vertex arrays that are
/// compatible with the engines rendering API.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_VERTEXARRAY_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_VERTEXARRAY_H_

#include <memory>

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/Buffer.h"

namespace lambda {
namespace core {
namespace renderer {

/// @brief The abstract VertexArray API.
///
/// Implemented by platform specific APIs.
class VertexArray {
 public:
  virtual ~VertexArray() = default;

  /// @brief Bind the VertexArray to the GPU.
  virtual void Bind() const = 0;

  /// @brief Unbind the VertexArray from the GPU.
  virtual void Unbind() const = 0;

  /// @brief Add a VertexBuffer to the current VertexArray.
  virtual void AddVertexBuffer(
      const memory::Shared<VertexBuffer>& vertex_buffer) = 0;

  /// @brief Set an IndexBuffer for the current VertexArray.
  virtual void SetIndexBuffer(
      const memory::Shared<IndexBuffer>& index_buffer) = 0;

  /// @brief Get the current IndexBuffer being used.
  virtual const memory::Shared<IndexBuffer>
      GetIndexBuffer() const = 0;

  /// @brief Get an array of VertexBuffers that are associated with the
  /// VertexArray.
  virtual const std::vector<memory::Shared<VertexBuffer>>
      GetVertexBuffers() const = 0;

  /// @brief Create a new VertexArray.
  static memory::Shared<VertexArray> Create();
};

}  // namespace renderer
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_VERTEXARRAY_H_
