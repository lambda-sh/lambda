#include "core/renderer/Buffer.h"

#include "core/Assert.h"
#include "core/renderer/Renderer.h"
#include "platform/opengl/OpenGLBuffer.h"

namespace engine {
namespace renderer {

VertexBuffer* VertexBuffer::Create(float* vertices, uint32_t size) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::None:
      ENGINE_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::OpenGL:
      return new platform::opengl::OpenGLVertexBuffer(vertices, size);
    default:
      ENGINE_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

IndexBuffer* IndexBuffer::Create(uint32_t* indices, uint32_t count) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::None:
      ENGINE_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::OpenGL:
      return new platform::opengl::OpenGLIndexBuffer(indices, count);
    default:
      ENGINE_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

}  // namespace renderer
}  // namespace engine
