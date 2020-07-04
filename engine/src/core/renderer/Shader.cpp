#include "core/renderer/Shader.h"

#include <string>
#include <vector>

#include <glad/glad.h>
#include <glm/gtc/type_ptr.hpp>

#include "core/util/Log.h"
#include "core/renderer/RendererAPI.h"
#include "core/renderer/Renderer.h"
#include "platform/opengl/OpenGLShader.h"

namespace engine {
namespace renderer {

Shader* Shader::Create(
    const std::string &vertex_source, const std::string &fragment_source) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      ENGINE_CORE_ASSERT(
          false, "There is no rendering API being used/available.");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return new platform::opengl::OpenGLShader(vertex_source, fragment_source);
    default:
      ENGINE_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.");
      return nullptr;
  }
}

}  // namespace renderer
}  // namespace engine
