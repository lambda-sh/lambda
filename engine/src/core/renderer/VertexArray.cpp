#include "core/renderer/VertexArray.h"

#include "core/util/Assert.h"
#include "core/renderer/Renderer.h"

#include "platform/opengl/OpenGLVertexArray.h"

namespace engine {
namespace renderer {

/**
 * Only supports Vertex buffers that are available through the rendering API.
 */
VertexArray* VertexArray::Create() {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      ENGINE_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return new engine::platform::opengl::OpenGLVertexArray();
    default:
      ENGINE_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

}  // namespace renderer
}  // namespace engine
