/// @file Buffer.h
/// @brief Buffer abstractions that allow the ease of implementing Buffers for
/// any graphics API.
///
/// All platform specific graphics API will implement buffer implementations in
/// their corresponding API syntax through this generalized engine API. This
/// will allow application developers to not have to worry (Not entirely true)
/// about platform specific buffer implementations.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_BUFFER_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_BUFFER_H_

#include <ostream>
#include <string>
#include <vector>

#include <Lambda/core/memory/Pointers.h>
#include <Lambda/lib/Assert.h>
#include <Lambda/lib/Log.h>

#include <Lambda/concepts/Point.h>

namespace lambda::core::renderer {

/// @brief Data types supported by the shader.
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

/// @brief Convert ShaderData types to their respective sizes in bytes.
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
    default: {
      LAMBDA_CORE_ASSERT(false, "Not a provided Shader type", ""); return 0;
    }
  }
}

/// @brief Obtain the component count from the shader type.
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
    default: {
      LAMBDA_CORE_ASSERT(false, "Not a provided Shader type", ""); return 0;
    }
  }
}

/// @brief A generic buffer element used for describing the layout of a buffer.
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
          Normalized(normalized) { LAMBDA_CORE_TRACE(ToString()) }

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

/// @brief The layout of a vertex buffer. Should always be instantiated with
/// buffer elements for it to work properly.
class BufferLayout {
 public:
  BufferLayout(const std::initializer_list<BufferElement>& elements)
    : elements_(elements) { CalculateOffsetAndStride(); }

  BufferLayout() {}

  /// @brief Get the stride
  uint32_t GetStride() const { return stride_; }

  /// @brief Get a reference to the list of the elements associated with the
  /// Buffer.
  const std::vector<BufferElement>& GetElements() const {
      return elements_; }

  /// @brief Checks to see if the BufferLayout has any elements associated with.
  const bool HasElements() const { return elements_.size() > 0; }

  std::vector<BufferElement>::iterator begin() { return elements_.begin(); }
  std::vector<BufferElement>::iterator end() { return elements_.end(); }

  std::vector<BufferElement>::const_iterator begin() const {
      return elements_.begin(); }
  std::vector<BufferElement>::const_iterator end() const {
      return elements_.end(); }

 private:
  std::vector<BufferElement> elements_;
  uint32_t stride_;

  /// @brief Calculates the offset per element and the stride for the overall
  /// buffer.
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

/// @brief A general abstraction of Vertex Buffer.
///
/// This should be constructed
class VertexBuffer {
 public:
  virtual ~VertexBuffer() {}

  /// @brief Binds a vertex buffer to the GPU.
  virtual void Bind() const = 0;

  /// @brief Unbinds a vertex buffer from the GPU. (Rarely needs to be used.)
  virtual void Unbind() const = 0;

  /// @brief Get the layout associated
  virtual const BufferLayout& GetLayout() const = 0;

  /// @brief Set the layout associated with the VertexBuffer.
  virtual void SetLayout(const BufferLayout&) = 0;

  /// @brief Create a Vertex buffer given a pointer to an array of vertices
  /// and the size of the buffer to be stored.
  ///
  /// While this returns a platform independent vertex buffer, it is still
  /// bound to a platform specific implementation under the hood.
  static memory::Shared<VertexBuffer> Create(float* vertices, uint32_t size);
};

/// @brief A general abstraction of an Index Buffer.
class IndexBuffer {
 public:
  virtual ~IndexBuffer() = default;

  /// @brief Binds the IndexBuffer to the GPU.
  virtual void Bind() const = 0;

  /// @brief Unbinds the IndexBuffer to the GPU.
  virtual void Unbind() const = 0;

  /// @brief Get the count
  virtual uint32_t GetCount() const = 0;

  /// @brief Create an IndexBuffer given a pointer to an array of indices and
  /// the count of indices in the array.
  ///
  /// While this returns a platform independent IndexBuffer, the IndexBuffer is
  /// still platform specific.
  static memory::Shared<IndexBuffer> Create(uint32_t* indices, uint32_t count);
};

}  // namespace lambda::core::renderer

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_BUFFER_H_
