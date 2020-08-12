#include "core/renderer/Texture.h"

#include "core/renderer/RendererAPI.h"
#include "core/renderer/Renderer.h"
#include "core/memory/Pointers.h"

#include "platform/opengl/OpenGLTexture.h"

namespace lambda {
namespace core {
namespace renderer {

memory::Shared<Texture2D> Texture2D::Create(const std::string& path) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      LAMBDA_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLTexture2D>(path);
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
