#include "Lambda/core/renderer/VertexArray.h"

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/lib/Assert.h"
#include "Lambda/core/renderer/Renderer.h"

#include "Lambda/platform/opengl/OpenGLVertexArray.h"

namespace lambda {
namespace core {
namespace renderer {

/**
 * Only supports Vertex buffers that are available through the rendering API.
 */
memory::Shared<VertexArray> VertexArray::Create() {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      LAMBDA_CORE_ASSERT(
          false, "There is no rendering API being used/available.", "");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLVertexArray>();
    default:
      LAMBDA_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.",
          "");
      return nullptr;
  }
}

}  // namespace renderer
}  // namespace core
}  // namespace lambda
