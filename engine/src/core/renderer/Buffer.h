/**
 * @file Buffer.h
 * @brief Buffer abstractions that allow the ease of implementing Buffers for
 * any graphics API.
 *
 * All platform specific graphics API will implement buffer implementations in
 * their corresponding API syntax through this generalized engine API. This will
 * allow application developers to not have to worry (Not entirely true) about
 * platform specific buffer implementations.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_BUFFER_H_
#define ENGINE_SRC_CORE_RENDERER_BUFFER_H_

#include <bits/stdint-uintn.h>
#include <ostream>
#include <string>
#include <vector>

#include "core/memory/Pointers.h"
#include "core/util/Assert.h"
#include "core/util/Log.h"

namespace engine {
namespace core {

namespace renderer {

enum class ShaderDataType {
  None = 0,
  Bool,
  Float,
  Float2,
  Float3,
  Float4,
  Int,
  Int2,
  Int3,
  Int4,
  Mat3,
  Mat4,
};

static uint32_t ShaderDataTypeSize(ShaderDataType type) {
  switch (type) {
    case ShaderDataType::Bool:  return 1;
    case ShaderDataType::Float: return 4;
    case ShaderDataType::Float2: return 4 * 2;
    case ShaderDataType::Float3: return 4 * 3;
    case ShaderDataType::Float4: return 4 * 4;
    case ShaderDataType::Int: return 4;
    case ShaderDataType::Int2: return 4 * 2;
    case ShaderDataType::Int3: return 4 * 3;
    case ShaderDataType::Int4: return 4 * 4;
    case ShaderDataType::Mat3: return 4 * 3 * 3;
    case ShaderDataType::Mat4: return 4 * 4 * 4;
    default: ENGINE_CORE_ASSERT(false, "Not a provided Shader type"); return 0;
  }
}

static uint32_t ShaderDataTypeComponentCount(ShaderDataType type) {
  switch (type) {
    case ShaderDataType::Bool:  return 1;
    case ShaderDataType::Float: return 1;
    case ShaderDataType::Float2: return 2;
    case ShaderDataType::Float3: return 3;
    case ShaderDataType::Float4: return 4;
    case ShaderDataType::Int: return 1;
    case ShaderDataType::Int2: return 2;
    case ShaderDataType::Int3: return 3;
    case ShaderDataType::Int4: return 4;
    case ShaderDataType::Mat3: return 3 * 3;
    case ShaderDataType::Mat4: return 4 * 4;
    default: ENGINE_CORE_ASSERT(false, "Not a provided Shader type"); return 0;
  }
}

struct BufferElement {
  ShaderDataType Type;
  std::string Name;
  uint32_t Size;
  uint32_t Offset;
  uint32_t Components;
  bool Normalized;

  BufferElement(
      ShaderDataType type, const std::string& name, bool normalized = false) :
          Type(type),
          Name(name),
          Size(ShaderDataTypeSize(type)),
          Offset(0),
          Components(ShaderDataTypeComponentCount(type)),
          Normalized(normalized) { ENGINE_CORE_TRACE(ToString()); }

  std::string ToString() const {
    return
        std::string("[Buffer Element] ")
        + "Name: " + Name
        + ", Offset: " + std::to_string(Offset)
        + ",  Size: " + std::to_string(Size)
        + ",  Components: " + std::to_string(Components)
        + ",  Normalized: " + std::to_string(Normalized);
  }

  inline std::ostream& operator<<(std::ostream& os) { return os << ToString(); }
};

class BufferLayout {
 public:
  BufferLayout(const std::initializer_list<BufferElement>& elements)
    : elements_(elements) { CalculateOffsetAndStride(); }

  BufferLayout() {}

  inline uint32_t GetStride() const { return stride_; }

  inline const std::vector<BufferElement>& GetElements() const {
      return elements_; }

  inline const bool HasElements() const { return elements_.size() > 0; }

  std::vector<BufferElement>::iterator begin() { return elements_.begin(); }
  std::vector<BufferElement>::iterator end() { return elements_.end(); }

  std::vector<BufferElement>::const_iterator begin() const {
      return elements_.begin(); }
  std::vector<BufferElement>::const_iterator end() const {
      return elements_.end(); }

 private:
  std::vector<BufferElement> elements_;
  uint32_t stride_;

  void CalculateOffsetAndStride() {
    uint32_t offset = 0;
    stride_ = 0;

    for (BufferElement& element : elements_) {
      element.Offset = offset;
      offset += element.Size;
      stride_ += element.Size;
    }
  }
};


class VertexBuffer {
 public:
  virtual ~VertexBuffer() {}

  virtual void Bind() const = 0;
  virtual void Unbind() const = 0;

  virtual const BufferLayout& GetLayout() const = 0;
  virtual void SetLayout(const BufferLayout&) = 0;

  static memory::Shared<VertexBuffer> Create(float* vertices, uint32_t size);
};

class IndexBuffer {
 public:
  virtual ~IndexBuffer() {}

  virtual void Bind() const = 0;
  virtual void Unbind() const = 0;

  virtual uint32_t GetCount() const = 0;

  static memory::Shared<IndexBuffer> Create(uint32_t* indices, uint32_t count);
};

}  // namespace renderer
}  // namespace core
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_BUFFER_H_

/**
 * @enum engine::renderer::ShaderDataType
 * @brief Data types that are compatible with shaders supported by the engine.
 */

/**
 * @fn engine::renderer::ShaderDataTypeSize
 * @brief Helper function that determines the size of a ShaderDataType in bytes.
 */

/**
 * @fn engine::renderer::ShaderDataTypeComponentCount
 * @brief Helper function that determines the number of components in a
 * ShaderDataType.
 */

/**
 * @struct engine::renderer::BufferElement
 * @brief A generic buffer element representation to be used in conjuction with
 * BufferLayouts.
 *
 * The creation of every buffer is logged if ENGINE_DEVELOPMENT_MODE is enabled
 * at compile time of of the engine. (Will disable in the future, but currently
 * still testing.)
 */

/**
 * @fn engine::renderer::BufferElement::BufferElement
 * @param type The ShaderDataType that should be sent into the graphics API.
 * @param name The name to be registered in the graphics API.
 */

/**
 * @class engine::renderer::BufferLayout
 * @brief A layout that specifies the elements to be associated with a vertex
 * buffer.
 */

/**
 * @fn engine::renderer::BufferLayout::BufferLayout
 * @brief Instantiate a BufferLayout with an initializer list of
 * BufferElements.
 *
 * Stride is calculated once a non empty elements e.g.
 * ```
 * renderer::BufferLayout layout_init_list = {
 *     { renderer::ShaderDataType::Float3, "a_Position"},
 *     { renderer::ShaderDataType::Float4, "a_Color"},
 *     { renderer::ShaderDataType::Float3, "a_Normal"}};
 *
 * renderer::BufferLayout layout(layout_init_list);
 * ```
 */

/**
 * @fn engine::renderer::BufferLayout::GetStride
 * @brief Get the overall stride of the current buffer layout.
 *
 * The stride is essentially the total size of the Buffer layout elements.
 */

/**
 * @fn engine::renderer::BufferLayout::GetElements
 * @brief Get a const reference to the elements associated with this layout.
 */

/**
 * @fn engine::renderer::BufferLayout::HasElements
 * @brief Checks to see if the current buffer layout has elements.
 */

/**
 * @class engine::renderer::VertexBuffer
 * @brief The base VertexBuffer class to be used for creating vertex buffers.
 *
 * Platform specific graphics API should extend this class in order to be
 * supported by the the rendering API.
 */

/**
 * @fn engine::renderer::VertexBuffer::Bind
 * @brief Bind the vertex buffer to the current rendering context.
 *
 * The binding process is entirely dependent upon the grahpics API. However,
 * it should be noted that this function needs to be called by the user
 * whenever they're trying to bind the buffer to the render API.
 */

/**
 * @fn engine::renderer::VertexBuffer::Unbind
 * @brief Unbind the vertex buffer to the current rendering context.
 *
 * The unbinding process is entirely dependent upon the grahpics API. However,
 * it should be noted that this function needs to be called by the user
 * whenever they're trying to unbind the buffer from the render API. (For
 * cleaning up anything still in memory.)
 */

/**
 * @fn engine::renderer::VertexBuffer::GetLayout
 * @brief Get the BufferLayout tied to the current VertexBuffer.
 */

/**
 * @fn engine::renderer::VertexBuffer::SetLayout
 * @brief Set the BufferLayout for the current VertexBuffer.
 */

/**
 * @fn engine::renderer::VertexBuffer::Create
 * @param vertices - a pointer to an array of vertices to be registered.
 * @param size - The size of the vertices in bytes.
 * @brief Creates a VertexBuffer through the Graphics API that is being used
 * at compile time.
 *
 * This is the primary method of creating platform independent vertex buffers
 * and what should be used by users to create Vertex Buffers that are
 * compatible with the rendering API.
 */

/**
 * @class engine::renderer::IndexBuffer
 * @brief The base IndexBuffer class to be used for creating index buffers.
 *
 * Platform specific graphics API should extend this class in order to be
 * supported by the the rendering API.
 */

/**
 * @fn engine::renderer::IndexBuffer::Bind
 * @brief Bind the current IndexBuffer to the current rendering context.
 */

/**
 * @fn engine::renderer::IndexBuffer::Unbind
 * @brief Unbind the current IndexBuffer from the current rendering context.
 *
 * Any vertex buffer that relies on this buffer will not be able to access
 * the contents of it once unbound from the graphics context.
 */

/**
 * @fn engine::renderer::IndexBuffer::GetCount
 * @brief Get the count of indices within the current IndexBuffer.
 */

/**
 * @fn engine::renderer::IndexBuffer::Create
 * @param indices - a pointer to an array of indices to be registered.
 * @param size - The size of the vertices in bytes.
 * @brief Creates a IndexBuffer through the Graphics API that is being used
 * at compile time.
 *
 * This is the primary method of creating platform independent index buffers
 * and what should be used by users to create Vertex Buffers that are
 * compatible with the rendering API.
 */
