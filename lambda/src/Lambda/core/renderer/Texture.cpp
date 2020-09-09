#include "Lambda/core/renderer/Texture.h"

#include "Lambda/core/renderer/RendererAPI.h"
#include "Lambda/core/renderer/Renderer.h"
#include "Lambda/core/memory/Pointers.h"

#include "Lambda/platform/opengl/OpenGLTexture.h"

namespace lambda {
namespace core {
namespace renderer {

memory::Shared<Texture2D> Texture2D::Create(uint32_t width, uint32_t height) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      LAMBDA_CORE_ASSERT(
          false, "There is no rendering API being used/available");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLTexture2D>(
          width, height);
    default:
      LAMBDA_CORE_ASSERT(
          false,
          "The renderer has been set to a graphics API that isn't supported");
      return nullptr;
  }
}

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
