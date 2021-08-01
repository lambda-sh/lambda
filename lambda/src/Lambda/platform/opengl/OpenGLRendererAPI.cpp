#include <Lambda/platform/opengl/OpenGLRendererAPI.h>


#include <Lambda/core/memory/Pointers.h>

#include <Lambda/platform/glad/Glad.h>

namespace lambda::platform::opengl {

namespace {

using core::memory::Shared;
using core::renderer::RendererAPI;

constexpr auto GetOpenGLPrimitive(const RendererAPI::Primitive primitive) {
  switch (primitive) {
    case RendererAPI::Primitive::Lines:
      return GL_LINES;
    case RendererAPI::Primitive::Triangles:
      return GL_TRIANGLES;
    case RendererAPI::Primitive::LineStrip:
      return GL_LINE_STRIP;
  }
  return 0;
}
}  // namespace

void OpenGLRendererAPI::Init() {
  glEnable(GL_BLEND);
  glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

  // Enables depth on a per pixel basis.
  glEnable(GL_DEPTH_TEST);
}

void OpenGLRendererAPI::SetClearColor(const glm::vec4& color) {
  glClearColor(color.r, color.g, color.b, color.a);
}

void OpenGLRendererAPI::SetViewport(
    uint32_t x, uint32_t y, uint32_t width, uint32_t height) {
  glViewport(x, y, width, height);
}

void OpenGLRendererAPI::Clear() {
  glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
}

/// TODO(C3NZ): Update this to use the engines memory system.
void OpenGLRendererAPI::DrawIndexed(
    Shared<core::renderer::VertexArray> vertex_array) {
  glDrawElements(
      GL_TRIANGLES,
      vertex_array->GetIndexBuffer()->GetCount(),
      GL_UNSIGNED_INT,
      nullptr);
}

void OpenGLRendererAPI::DrawArrays(core::memory::Shared<core::renderer::VertexArray> vertex_array) {
  glDrawArrays(GL_TRIANGLES, 0, vertex_array->GetVertexBuffers().size());
}

void OpenGLRendererAPI::DrawArrays(
    Shared<core::renderer::VertexArray> vertex_array,
    const Primitive primitive) {
  LAMBDA_CORE_TRACE("Drawing vertex array with size of: {0}", vertex_array->GetVertexBuffers().size());
  glDrawArrays(GetOpenGLPrimitive(primitive), 0, vertex_array->GetVertexBuffers().size());
}

}  // namespace lambda::platform::opengl
