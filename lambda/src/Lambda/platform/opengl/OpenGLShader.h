/// @file OpenGLShader.h
/// @brief Shader API to be used with the renderer.
#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLSHADER_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLSHADER_H_

#include <string>
#include <unordered_map>

#include <glad/glad.h>
#include <glm/glm.hpp>

#include "Lambda/core/renderer/Shader.h"

namespace lambda {
namespace platform {
namespace opengl {

/// @brief The Shader API.
///
/// Expects sources to be strings that are designed for the shading language
/// that is being used in the API. This can also load shaders stored within a
/// glsl file and conforms to the way that the Shader API expects to read them.
/// Look into the sandbox tool for working examples.
class OpenGLShader : public core::renderer::Shader {
 public:
  OpenGLShader(const std::string& path);
  OpenGLShader(
      const std::string& name,
      const std::string& vertex_source,
      const std::string& fragment_source);

  ~OpenGLShader();

  void Bind() const override;
  void Unbind() const override;
  void SetMat4(const std::string& name, const glm::mat4& matrix) override;
  void SetFloat3(const std::string& name, const glm::vec3& vector) override;
  void SetFloat4(const std::string& name, const glm::vec4& vector) override;

  // ---------------------------- OPENGL SPECIFIC ------------------------------

  /// @brief Upload an integer into the shader.
  void UploadUniformInt(const std::string& name, const int& value);

  /// @brief Upload a float into the shader.
  void UploadUniformFloat(const std::string& name, const float& value);

  /// @brief Upload a uniform of 2 floats into the shader.
  void UploadUniformFloat2(const std::string& name, const glm::vec2& values);

  /// @brief Upload a uniform of 3 floats into the shader.
  void UploadUniformFloat3(const std::string& name, const glm::vec3& values);

  /// @brief Allows the uploading of uniform float 4s into the shader.
  void UploadUniformFloat4(const std::string& name, const glm::vec4& matrix);

  /// @brief Upload a uniform of Matrix 3s into the shader.
  void UploadUniformMat3(const std::string& name, const glm::mat3& matrix);

  /// @brief Upload a uniform of Matrix 4s into the shader.
  void UploadUniformMat4(const std::string& name, const glm::mat4& matrix);
 private:
  std::string ReadFile(const std::string& path);
  std::unordered_map<GLenum, std::string> PreProcess(
      const std::string& shader_source);
  void Compile(
      const std::unordered_map<GLenum, std::string>& shader_source_map);

};

}  // namespace opengl
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLSHADER_H_
