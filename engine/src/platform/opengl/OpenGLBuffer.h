#ifndef ENGINE_SRC_PLATFORM_OPENGL_OPENGLBUFFER_H_
#define ENGINE_SRC_PLATFORM_OPENGL_OPENGLBUFFER_H_

#include <bits/stdint-uintn.h>

#include "core/renderer/Buffer.h"

namespace engine {
namespace platform {
namespace opengl {


// ----------------------------- VERTEX BUFFER IMPL ----------------------------

class OpenGLVertexBuffer : public renderer::VertexBuffer {
 public:
  OpenGLVertexBuffer(float* vertices, uint32_t size);
  ~OpenGLVertexBuffer();

  void Bind() const override;
  void Unbind() const override;

 private:
  uint32_t renderer_ID_;
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
