#include "core/renderer/Buffer.h"

#include "core/memory/Pointers.h"
#include "core/renderer/Renderer.h"
#include "core/renderer/RendererAPI.h"
#include "core/util/Assert.h"
#include "platform/opengl/OpenGLBuffer.h"

namespace engine {
namespace renderer {

memory::Shared<VertexBuffer> VertexBuffer::Create(
    float* vertices, uint32_t size) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      ENGINE_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLVertexBuffer>(
          vertices, size);
    default:
      ENGINE_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

memory::Shared<IndexBuffer> IndexBuffer::Create(
    uint32_t* indices, uint32_t count) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      ENGINE_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLIndexBuffer>(
          indices, count);
    default:
      ENGINE_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

}  // namespace renderer
}  // namespace engine
