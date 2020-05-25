#ifndef ENGINE_SRC_CORE_RENDERER_BUFFER_H_
#define ENGINE_SRC_CORE_RENDERER_BUFFER_H_

#include <bits/stdint-uintn.h>
#include <string>
#include <ostream>
#include <vector>

#include "core/Assert.h"
#include "core/Log.h"

namespace engine {
namespace renderer {

/**
 * A list of data types that are compatible with shaders used in the engine.
 */
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

/**
 * Helper function that determines the size of a ShaderDataType in bytes.
 */
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

/**
 * Helper function that determines the number of components in a ShaderDataType.
 */
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

/**
 * A generic buffer element that allows contexts to access. These components are
 * to be used in the process of creatintg a BufferLayout.
 *
 * The creation of every buffer is logged if ENGINE_DEVELOPMENT_MODE is enabled
 * at compile time of of the engine.
 */
struct BufferElement {
  ShaderDataType Type;
  std::string Name;
  uint32_t Size;
  uint32_t Offset;
  uint32_t Components;
  bool Normalized;

  /**
   * Create a buffer element with a shader type and variable name to be used by
   * the current graphics context shader API.
   */
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
};

inline std::ostream& operator<<(std::ostream& os, const BufferElement& element)
    { return os << element.ToString(); }

/**
 * The layout for a given vertex buffer for creating shaders to be run by
 * the gpu.
 */
class BufferLayout {
 public:
  /**
   * Instantiate a BufferLayout with an initializer list of BufferElements.
   * e.g.
   * ```c++
   *   renderer::BufferLayout layout_init_list = {
   *       { renderer::ShaderDataType::Float3, "a_Position"},
   *       { renderer::ShaderDataType::Float4, "a_Color"},
   *       { renderer::ShaderDataType::Float3, "a_Normal"}};
   *
   *   renderer::BufferLayout layout(layout_init_list);
   * ```
   */
  BufferLayout(const std::initializer_list<BufferElement>& elements)
    : elements_(elements) { CalculateOffsetAndStride(); }


  /**
   * Instantiate an empty BufferLayout.
   */
  BufferLayout() {}

  inline uint32_t GetStride() const { return stride_; }

  inline const std::vector<BufferElement>& GetElements() const
    { return elements_; }

  std::vector<BufferElement>::iterator begin() { return elements_.begin(); }
  std::vector<BufferElement>::iterator end() { return elements_.end(); }

 private:
  std::vector<BufferElement> elements_;
  uint32_t stride_;

  /**
   * Computes the offset and stride for all of the BufferElements being stored.
   */
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


/**
 * Generic VertexBuffer implementation to represent generalized Vertex buffers
 * to be implemented for platform specific graphics libraries.
 */
class VertexBuffer {
 public:
  virtual ~VertexBuffer() {}

  virtual void Bind() const = 0;
  virtual void Unbind() const = 0;

  virtual const BufferLayout& GetLayout() const = 0;
  virtual void SetLayout(const BufferLayout&) = 0;

  static VertexBuffer* Create(float* vertices, uint32_t size);
};

/**
 * Generic IndexBuffer implementation to represent generalized Vertex buffers
 * to be implemented for platform specific graphics libraries.
 */
class IndexBuffer {
 public:
  virtual ~IndexBuffer() {}

  virtual void Bind() const = 0;
  virtual void Unbind() const = 0;

  virtual uint32_t GetCount() const = 0;

  static IndexBuffer* Create(uint32_t* indices, uint32_t count);
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_BUFFER_H_
