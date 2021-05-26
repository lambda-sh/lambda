#include <Lambda/core/renderer/Shader.h>

#include <string>

#include <Lambda/core/memory/Pointers.h>
#include <Lambda/core/renderer/Renderer.h>
#include <Lambda/core/renderer/RendererAPI.h>
#include <Lambda/lib/Log.h>

#include <Lambda/platform/opengl/OpenGLShader.h>
#include <Lambda/platform/glad/Glad.h>

namespace lambda::core::renderer {

/// @todo(C3NZ): Convert this into std::string_view
memory::Shared<Shader> Shader::Create(const std::string& path) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      LAMBDA_CORE_ASSERT(
          false, "There is no rendering API being used/available.", "");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLShader>(path);
    default:
      LAMBDA_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.",
          "");
      return nullptr;
  }
}

/// @todo(C3NZ): Convert this into std::string_view
memory::Shared<Shader> Shader::Create(
    const std::string& name,
    const std::string& vertex_source,
    const std::string& fragment_source) {
  switch (Renderer::GetAPI()) {
    case RendererAPI::API::None:
      LAMBDA_CORE_ASSERT(
          false, "There is no rendering API being used/available.", "");
      return nullptr;
    case RendererAPI::API::OpenGL:
      return memory::CreateShared<platform::opengl::OpenGLShader>(
          name, vertex_source, fragment_source);
    default:
      LAMBDA_CORE_ASSERT(
          false,
          "The Renderer has been set to a graphics API that isn't supported.",
          "");
      return nullptr;
  }
}

/// @todo(C3NZ): Do not pass around a shared pointer to the shader.
void ShaderLibrary::Add(const memory::Shared<Shader>& shader) {
  const std::string& name = shader->GetName();
  LAMBDA_CORE_ASSERT(
      shader_mapping_.find(name) == shader_mapping_.end(),
      "Shader is already stored within the engine.",
      "");

  shader_mapping_[name] = shader;
  LAMBDA_CORE_TRACE("Added the shader: {0}", name);
}

/// @todo(C3NZ): Do not pass around a shared pointer to the shader.
/// @todo(C3NZ): Use std::stringview instead and copy data as key?
void ShaderLibrary::Add(
    const std::string& name, const memory::Shared<Shader>& shader) {
  LAMBDA_CORE_ASSERT(
      shader_mapping_.find(name) == shader_mapping_.end(),
      "Shader is already stored within the engine.",
      "");

  shader_mapping_[name] = shader;
  LAMBDA_CORE_TRACE("Added the shader: {0}", name);
}

memory::Shared<Shader> ShaderLibrary::Load(const std::string& path) {
  memory::Shared<Shader> shader = Shader::Create(path);
  Add(shader);
  return shader;
}

memory::Shared<Shader> ShaderLibrary::Load(
    const std::string& name, const std::string& path) {
  memory::Shared<Shader> shader = Shader::Create(path);
  Add(name, shader);
  return shader;
}


memory::Shared<Shader> ShaderLibrary::Get(const std::string& name) {
  LAMBDA_CORE_ASSERT(
      shader_mapping_.find(name) != shader_mapping_.end(),
      "Failed to get the shader: {0}",
      name);
  return shader_mapping_[name];
}

}  // namespace lambda::core::renderer
