#include "Lambda/core/renderer/Buffer.h"

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/Renderer.h"
#include "Lambda/core/renderer/RendererAPI.h"
#include "Lambda/core/util/Assert.h"
#include "Lambda/platform/opengl/OpenGLBuffer.h"

namespace lambda {
namespace core {
namespace renderer {

memory::Shared<VertexBuffer> VertexBuffer::Create(
    float* vertices, uint32_t size) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      LAMBDA_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLVertexBuffer>(
          vertices, size);
    default:
      LAMBDA_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

memory::Shared<IndexBuffer> IndexBuffer::Create(
    uint32_t* indices, uint32_t count) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      LAMBDA_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLIndexBuffer>(
          indices, count);
    default:
      LAMBDA_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

}  // namespace renderer
}  // namespace core
}  // namespace lambda
