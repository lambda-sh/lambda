#include "platform/opengl/OpenGLRendererAPI.h"
#include <glad/glad.h>

namespace engine {
namespace platform {
namespace opengl {

void OpenGLRendererAPI::SetClearColor(const glm::vec4& color) {
  glClearColor(color.r, color.g, color.b, color.a);
}

void OpenGLRendererAPI::Clear() {
  glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
}

void OpenGLRendererAPI::DrawIndexed(
    const std::shared_ptr<renderer::VertexArray>& vertex_array) {
  glDrawElements(
      GL_TRIANGLES,
      vertex_array->GetIndexBuffer()->GetCount(),
      GL_UNSIGNED_INT,
      nullptr);
}


}  // namespace opengl
}  // namespace platform
}  // namespace engine