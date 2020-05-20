#ifndef ENGINE_SRC_CORE_RENDERER_BUFFER_H_
#define ENGINE_SRC_CORE_RENDERER_BUFFER_H_

namespace engine {
namespace renderer {

class VertexBuffer {
 public:
  virtual ~VertexBuffer() {}

  virtual void Bind() = 0;
  virtual void Unbind() = 0;
};

class IndexBuffer {
 public:
  virtual ~IndexBuffer() {}
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_BUFFER_H_
