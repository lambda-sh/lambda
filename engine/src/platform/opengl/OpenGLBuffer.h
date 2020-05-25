#ifndef ENGINE_SRC_PLATFORM_OPENGL_OPENGLBUFFER_H_
#define ENGINE_SRC_PLATFORM_OPENGL_OPENGLBUFFER_H_

#include <bits/stdint-uintn.h>

#include "core/renderer/Buffer.h"

namespace engine {
namespace platform {
namespace opengl {


// ----------------------------- VERTEX BUFFER IMPL ----------------------------

/**
 * The OpenGL VertexBuffer  implementation based off the generic
 * VertexBuffer base class provided by the engines renderer.
 */
class OpenGLVertexBuffer : public renderer::VertexBuffer {
 public:
  OpenGLVertexBuffer(float* vertices, uint32_t size);
  ~OpenGLVertexBuffer();

  /**
   * Create and bind the vertex buffer on the GPU. Will set the renderer_ID_.
   */
  void Bind() const override;

  /**
   * Unbind and delete the vertex buffer on GPU using the renderer_ID_
   * associated with it.
   */
  void Unbind() const override;

  /**
   * Get the BufferLayout associated with the current VertexBuffer.
   */
  const renderer::BufferLayout& GetLayout() const override { return layout_; };

  /**
   * Set the BufferLayout associated with the current VertexBuffer.
   */
  void SetLayout(const renderer::BufferLayout& layout) override
    { layout_ = layout; };

 private:
  uint32_t renderer_ID_;
  renderer::BufferLayout layout_;

};

// ----------------------------- INDEX BUFFER IMPL -----------------------------

class OpenGLIndexBuffer : public renderer::IndexBuffer {
 public:
   OpenGLIndexBuffer(uint32_t* indices, uint32_t count);
   ~OpenGLIndexBuffer();

   void Bind() const override;
   void Unbind() const override;

   inline uint32_t GetCount() const override { return count_; }

 private:
  uint32_t count_;
  uint32_t renderer_ID_;
};


}  // namespace opengl
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_SRC_PLATFORM_OPENGL_OPENGLBUFFER_H_
