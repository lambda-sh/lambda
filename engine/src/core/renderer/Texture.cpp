#include "core/renderer/Texture.h"

#include "core/renderer/RendererAPI.h"
#include "core/renderer/Renderer.h"
#include "core/memory/Pointers.h"

#include "platform/opengl/OpenGLTexture.h"

namespace engine {
namespace renderer {

memory::Shared<Texture2D> Texture2D::Create(const std::string& path) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      ENGINE_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLTexture2D>(path);
    default:
      ENGINE_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

}  // namespace renderer
}  // namespace engine
